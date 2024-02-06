use egui::CursorIcon;
use x11rb::{
    connection::Connection as _,
    protocol::xproto::{ChangeWindowAttributesAux, ConnectionExt},
    resource_manager::Database,
    xcb_ffi::{ReplyOrIdError, XCBConnection},
};

pub fn handle(
    conn: &XCBConnection, db: &Database, screen_num: usize, window: u32, cursor_icon: CursorIcon,
) {
    match handle_impl(conn, db, screen_num, window, cursor_icon) {
        Ok(_) => {}
        Err(e) => {
            println!("Failed to set cursor: {:?}", e);
        }
    }
}

fn handle_impl(
    conn: &XCBConnection, db: &Database, screen_num: usize, window: u32, cursor_icon: CursorIcon,
) -> Result<(), ReplyOrIdError> {
    let cursor_handle = x11rb::cursor::Handle::new(conn, screen_num, db)?.reply()?;
    let cursor = cursor_handle.load_cursor(conn, to_x11_cursor(cursor_icon))?;

    conn.change_window_attributes(window, &ChangeWindowAttributesAux::default().cursor(cursor))?
        .check()?;
    conn.flush()?;

    Ok(())
}

// Generated by ChatGPT with some values checked against my system's cursor icons at /usr/share/icons/<theme>/cursors
fn to_x11_cursor(cursor_icon: CursorIcon) -> &'static str {
    match cursor_icon {
        CursorIcon::Default => "left_ptr", // Default arrow cursor
        CursorIcon::None => "none",        // Hidden cursor
        CursorIcon::ContextMenu => "context-menu",
        CursorIcon::Help => "help",
        CursorIcon::PointingHand => "pointer",
        CursorIcon::Progress => "progress",
        CursorIcon::Wait => "wait",
        CursorIcon::Cell => "cell",
        CursorIcon::Crosshair => "crosshair",
        CursorIcon::Text => "text",
        CursorIcon::VerticalText => "vertical-text",
        CursorIcon::Alias => "alias",
        CursorIcon::Copy => "copy",
        CursorIcon::Move => "move",
        CursorIcon::NoDrop => "no-drop",
        CursorIcon::NotAllowed => "not-allowed",
        CursorIcon::Grab => "grab",
        CursorIcon::Grabbing => "grabbing",
        CursorIcon::AllScroll => "all-scroll",
        CursorIcon::ResizeHorizontal => "resize-horiz",
        CursorIcon::ResizeNeSw => "nesw-resize",
        CursorIcon::ResizeNwSe => "nwse-resize",
        CursorIcon::ResizeVertical => "vertical-resize",
        CursorIcon::ResizeEast => "e-resize",
        CursorIcon::ResizeSouthEast => "se-resize",
        CursorIcon::ResizeSouth => "s-resize",
        CursorIcon::ResizeSouthWest => "sw-resize",
        CursorIcon::ResizeWest => "w-resize",
        CursorIcon::ResizeNorthWest => "nw-resize",
        CursorIcon::ResizeNorth => "n-resize",
        CursorIcon::ResizeNorthEast => "ne-resize",
        CursorIcon::ResizeColumn => "col-resize",
        CursorIcon::ResizeRow => "row-resize",
        CursorIcon::ZoomIn => "zoom-in",
        CursorIcon::ZoomOut => "zoom-out",
    }
}
