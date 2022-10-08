use slot_vec::graph::NodeId;
use std::time::Instant;

use crate::widget::ComponentIndex;

#[derive(Debug)]
pub enum InternalMessage<Message> {
    Quit,
    RequestUpdate(NodeId, ComponentIndex),
    RequestRender(Instant),
    Broadcast(Message),
}
