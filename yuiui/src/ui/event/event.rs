use super::window_event::WindowEvent;

#[derive(Debug)]
pub enum Event<Message, WindowId> {
    LoopInitialized,
    Message(Message),
    WindowEvent(WindowId, WindowEvent),
}
