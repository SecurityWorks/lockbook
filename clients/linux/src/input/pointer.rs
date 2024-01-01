use lbeguiapp::WgpuLockbook;
use x11rb::protocol::xproto::{ButtonPressEvent, KeyButMask, MotionNotifyEvent};

use super::modifiers;

pub fn handle_press(app: &mut WgpuLockbook, event: ButtonPressEvent) {
    handle(app, event.event_x, event.event_y, event.detail, event.state, true)
}

pub fn handle_release(app: &mut WgpuLockbook, event: ButtonPressEvent) {
    handle(app, event.event_x, event.event_y, event.detail, event.state, false)
}

// written with reference to winit:
// https://github.com/rust-windowing/winit/blob/ca1674519ab3d8df4ce231fe018196a3981c7dea/src/platform_impl/linux/x11/event_processor.rs#L762
fn handle(
    app: &mut WgpuLockbook, event_x: i16, event_y: i16, detail: u8, state: KeyButMask,
    pressed: bool,
) {
    let modifiers = modifiers(state);

    if 4 <= detail && detail <= 7 {
        // scroll event
        // todo: also send mouse wheel event
        let scroll_unit = 10.0;
        let delta = match detail {
            4 if modifiers.shift => egui::Vec2::new(scroll_unit, 0.0),
            5 if modifiers.shift => egui::Vec2::new(-scroll_unit, 0.0),
            4 => egui::Vec2::new(0.0, scroll_unit),
            5 => egui::Vec2::new(0.0, -scroll_unit),
            6 => egui::Vec2::new(scroll_unit, 0.0),
            7 => egui::Vec2::new(-scroll_unit, 0.0),
            _ => unreachable!(),
        };
        app.raw_input.events.push(egui::Event::Scroll(delta))
    } else {
        // button event
        let pos = egui::Pos2::new(event_x as f32, event_y as f32);
        let button = match detail {
            1 => egui::PointerButton::Primary,
            2 => egui::PointerButton::Middle,
            3 => egui::PointerButton::Secondary,
            8 => egui::PointerButton::Extra1, // back
            9 => egui::PointerButton::Extra2, // forward
            _ => return,
        };
        app.raw_input
            .events
            .push(egui::Event::PointerButton { pos, button, pressed, modifiers })
    }
}

pub fn handle_motion(app: &mut WgpuLockbook, event: MotionNotifyEvent) {
    let pos = egui::Pos2::new(event.event_x as f32, event.event_y as f32);
    app.raw_input.events.push(egui::Event::PointerMoved(pos));
}
