use yuiui_support::bit_flags::BitFlags;

use crate::event::WindowEventMask;
use super::command::Command;

pub enum Effect<Message> {
    AddListener(BitFlags<WindowEventMask>),
    RemoveListener(BitFlags<WindowEventMask>),
    Command(Command<Message>),
}

