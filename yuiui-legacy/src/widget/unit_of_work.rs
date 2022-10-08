use slot_vec::graph::NodeId;

use super::RcWidget;

#[derive(Debug)]
pub enum UnitOfWork<State, Message> {
    Append(NodeId, RcWidget<State, Message>),
    Insert(NodeId, RcWidget<State, Message>),
    Replace(NodeId, RcWidget<State, Message>),
    Update(NodeId, RcWidget<State, Message>),
    Move(NodeId, NodeId),
    Remove(NodeId),
    RemoveChildren(NodeId),
}
