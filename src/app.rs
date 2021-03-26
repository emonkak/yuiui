use std::collections::HashMap;

use x11::xlib;

use context::Context;
use context::Event;
use event_handler::EventHandler;
use tray::Tray;
use tray::TrayIcon;

pub fn run(context: Context) {
    let mut tray = Tray::new(context.display);
    tray.acquire_tray_selection();
    tray.show();

    let mut icons: HashMap<xlib::Window, TrayIcon> = HashMap::new();

    unsafe {
        xlib::XFlush(context.display);
    }

    context.poll_events(|event| {
        match event {
            Event::XEvent(event) => tray.handle_event(event),
            Event::Signal(_) => false,
        }
    });
}
