use std::time::Instant;
use yuiui_support::slot_tree::NodeId;

use crate::widget::ComponentIndex;

#[derive(Debug)]
pub enum ApplicationMessage<Message> {
    Quit,
    RequestUpdate(NodeId, ComponentIndex),
    Render(Instant),
    Broadcast(Message),
}
