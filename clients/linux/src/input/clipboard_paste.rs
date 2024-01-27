use lb::FileType;
use lbeguiapp::WgpuLockbook;
use std::mem;
use std::time::{SystemTime, UNIX_EPOCH};
use x11rb::protocol::xproto::{Atom, ConnectionExt as _};
use x11rb::reexports::x11rb_protocol::protocol::xproto;
use x11rb::xcb_ffi::XCBConnection;

use crate::window::AtomCollection;

pub struct Context<'a> {
    ctx: Ctx<'a>,
    state: State,
}

#[derive(Clone)]
struct Ctx<'a> {
    window: xproto::Window,
    conn: &'a XCBConnection,
    atoms: &'a AtomCollection,
}

#[derive(PartialEq)]
enum State {
    AwaitingPaste,
    AwaitingTargets,
    AwaitingData { source_window: xproto::Window, format: Atom },
    AwaitingIncrementalData { incremental_data: Vec<u8>, format: Atom },
}

impl<'a> Context<'a> {
    pub fn new(window: xproto::Window, conn: &'a XCBConnection, atoms: &'a AtomCollection) -> Self {
        Self { ctx: Ctx { window, conn, atoms }, state: State::AwaitingPaste }
    }

    pub fn handle_paste(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let Ctx { window, conn, atoms } = self.ctx;

        // request clipboard targets; we'll get a SelectionNotify event when it's ready
        conn.convert_selection(
            window,
            atoms.CLIPBOARD,
            atoms.TARGETS,
            atoms.CLIPBOARD,
            x11rb::CURRENT_TIME,
        )?;
        self.state = State::AwaitingTargets;

        Ok(())
    }

    pub fn handle_selection_notify(
        &mut self, event: &xproto::SelectionNotifyEvent, app: &mut WgpuLockbook,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let Ctx { conn, atoms, window } = self.ctx;

        // todo: support middle mouse button paste
        if event.property == atoms.CLIPBOARD {
            if let State::AwaitingTargets = self.state {
                // get supported formats
                if event.target != atoms.TARGETS {
                    println!("handle selection: awaiting targets but received non-targets");
                    return Ok(());
                }
                if event.requestor != window {
                    println!("handle selection: received targets from wrong window");
                    return Ok(());
                }
                let targets = self.ctx.get_targets()?;

                // select a format
                let format = if targets.contains(&atoms.ImagePng) {
                    atoms.ImagePng
                } else if targets.contains(&atoms.UTF8_STRING) {
                    atoms.UTF8_STRING
                } else {
                    // no supported formats available
                    println!("handle selection: no supported formats available");
                    self.state = State::AwaitingPaste;
                    return Ok(());
                };

                // request clipboard data in the format; we'll get another SelectionNotifyEvent when it's ready
                conn.convert_selection(
                    event.requestor,
                    atoms.CLIPBOARD,
                    format,
                    atoms.CLIPBOARD,
                    event.time,
                )?;
                self.state = State::AwaitingData { source_window: event.requestor, format };
            } else if let State::AwaitingData { source_window, format } = &self.state {
                let format = *format;

                if event.target != format {
                    let event_name = conn.get_atom_name(event.target)?.reply()?.name;
                    let event_name = String::from_utf8(event_name).expect("get atom name as utf8");
                    let expected_target = conn.get_atom_name(format)?.reply()?.name;
                    let expected_target =
                        String::from_utf8(expected_target).expect("get atom name as utf8");
                    println!(
                        "handle selection: received data of wrong type: {} (expected {})",
                        event_name, expected_target
                    );
                    return Ok(());
                }
                if event.requestor != *source_window {
                    println!("handle selection: received data from wrong window");
                    return Ok(());
                }

                // get clipboard data
                if let Some(data) = self.ctx.read_clipboard_data(format)? {
                    self.ctx.app_paste(format, data, app)?;
                    self.state = State::AwaitingPaste;
                } else {
                    // data is being transferred incrementally; delete the property to initiate the transfer
                    conn.delete_property(event.requestor, atoms.CLIPBOARD)?;
                    self.state =
                        State::AwaitingIncrementalData { incremental_data: Vec::new(), format };
                }
            }
        }

        Ok(())
    }

    pub fn handle_property_notify(
        &mut self, event: &xproto::PropertyNotifyEvent, app: &mut WgpuLockbook,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let Ctx { conn, atoms, .. } = self.ctx;

        let mut done = false;
        if let State::AwaitingIncrementalData { incremental_data, format } = &mut self.state {
            let format = *format;

            if let Some(data) = self.ctx.read_clipboard_data(format)? {
                incremental_data.extend_from_slice(&data);
                if !data.is_empty() {
                    // delete the property to continue the transfer
                    conn.delete_property(event.window, atoms.CLIPBOARD)?;
                } else {
                    // empty increment means the transfer is complete
                    done = true;
                }
            }
        }

        if done {
            if let State::AwaitingIncrementalData { incremental_data, format } =
                mem::replace(&mut self.state, State::AwaitingPaste)
            {
                if !incremental_data.is_empty() {
                    self.ctx.app_paste(format, incremental_data, app)?;
                }
            }
        }

        Ok(())
    }
}

impl<'a> Ctx<'a> {
    fn get_targets(&self) -> Result<Vec<Atom>, Box<dyn std::error::Error>> {
        let Ctx { conn, atoms, window } = *self;
        let formats = conn
            .get_property(
                false,
                window,
                atoms.CLIPBOARD,
                xproto::Atom::from(xproto::AtomEnum::ATOM),
                0,
                std::u32::MAX,
            )?
            .reply()?
            .value;

        let mut types = Vec::new();
        for atom in formats.chunks_exact(4) {
            let atom = xproto::Atom::from_ne_bytes([atom[0], atom[1], atom[2], atom[3]]);
            types.push(atom);
        }
        Ok(types)
    }

    /// Reads clipboard data of the given type from the given window. Returns None if the data is being transferred
    /// incrementally. Otherwise, returns the data.
    fn read_clipboard_data(
        &self, format: Atom,
    ) -> Result<Option<Vec<u8>>, Box<dyn std::error::Error>> {
        let Ctx { window, conn, atoms } = *self;

        let mut data = Vec::new();
        let mut offset = 0;
        let mut incr = false;
        loop {
            let reply = conn
                .get_property(false, window, atoms.CLIPBOARD, format, offset, std::u32::MAX)?
                .reply()?;

            incr |= reply.type_ == atoms.INCR;
            if incr {
                return Ok(None);
            }

            data.extend_from_slice(&reply.value);
            offset += reply.value_len;
            if reply.bytes_after == 0 {
                break;
            }
        }

        Ok(Some(data))
    }

    // todo: dedupe with code in windows app, possibly other places
    fn app_paste(
        &mut self, format: Atom, data: Vec<u8>, app: &mut WgpuLockbook,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let Ctx { conn, atoms, .. } = *self;

        if format == atoms.UTF8_STRING {
            // utf8 -> egui paste event
            let text = String::from_utf8_lossy(&data);
            app.raw_input
                .events
                .push(egui::Event::Paste(text.to_string()));
        } else if format == atoms.ImagePng {
            // png -> import lockbook file and paste markdown image link
            let core = match &app.app {
                lbeguiapp::Lockbook::Splash(_) => {
                    return Ok(());
                }
                lbeguiapp::Lockbook::Onboard(screen) => &screen.core,
                lbeguiapp::Lockbook::Account(screen) => &screen.core,
            };
            let file = core
                .create_file(
                    &format!(
                        "pasted_image_{}.png",
                        SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap()
                            .as_micros()
                    ),
                    core.get_root().expect("get lockbook root").id,
                    FileType::Document,
                )
                .expect("create lockbook file for image");
            core.write_document(file.id, &data)
                .expect("write lockbook file for image");
            let markdown_image_link = format!("![pasted image](lb://{})", file.id);
            app.raw_input
                .events
                .push(egui::Event::Paste(markdown_image_link));
        } else {
            // unsupported type -> print and ignore
            let name = conn.get_atom_name(format)?.reply()?.name;
            let name = String::from_utf8(name).expect("get atom name as utf8");
            println!("handle selection: unsupported clipboard type: {}", name);
        }

        Ok(())
    }
}
