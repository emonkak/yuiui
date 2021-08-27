pub mod mouse;
pub mod window;

mod event;
mod handler;
mod manager;

pub use event::{EventType, GenericEvent};
pub use handler::EventHandler;
pub use manager::{EventManager, EventHandlerId};
