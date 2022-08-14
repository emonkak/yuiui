use std::time::Instant;
use yuiui_support::slot_tree::NodeId;

use crate::widget::ComponentIndex;

#[derive(Debug)]
pub enum InternalMessage<Message> {
    Quit,
    RequestUpdate(NodeId, ComponentIndex),
    RequestRender(Instant),
    Broadcast(Message),
}
