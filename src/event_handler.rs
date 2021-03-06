use x11::xlib;

pub trait EventHandler {
    fn handle_event(&mut self, event: xlib::XEvent) -> bool {
        match event.get_type() {
            xlib::ClientMessage => self.handle_client_message(xlib::XClientMessageEvent::from(event)),
            xlib::DestroyNotify => self.handle_destroy_notify(xlib::XDestroyWindowEvent::from(event)),
            xlib::ReparentNotify => self.handle_reparent_notify(xlib::XReparentEvent::from(event)),
            _ => true,
        }
    }

    fn handle_client_message(&mut self, event: xlib::XClientMessageEvent) -> bool;

    fn handle_destroy_notify(&mut self, event: xlib::XDestroyWindowEvent) -> bool;

    fn handle_reparent_notify(&mut self, event: xlib::XReparentEvent) -> bool;
}
