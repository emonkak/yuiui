use context::Context;
use context::Event;
use tray::Tray;

pub fn run(context: Context) {
    let mut context = context;
    let mut tray = Tray::new(&context);

    let previous_selection_owner = context.acquire_tray_selection(tray.window);

    tray.show();

    context.poll_events(|context, event| {
        match event {
            Event::XEvent(event) => tray.handle_event(context, event),
            Event::Signal(signal) => {
                println!("{:?}", signal);
                false
            },
        }
    });

    context.release_tray_selection(previous_selection_owner);
}
