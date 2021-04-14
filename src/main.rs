extern crate keytray;

use std::env;

use keytray::config::Config;
use keytray::context::Context;
use keytray::context::Event;
use keytray::tray::Tray;

fn main() {
    let args = env::args().collect();
    let config = Config::parse(args);

    let context = Context::new(config).unwrap();
    let mut tray = Tray::new(&context);

    let previous_selection_owner = context.acquire_tray_selection(tray.window());

    tray.show();

    context.poll_events(|event| {
        match event {
            Event::XEvent(event) => tray.on_event(event),
            Event::Signal(_) => false,
        }
    });

    context.release_tray_selection(previous_selection_owner);
}
