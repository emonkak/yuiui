use std::rc::Rc;
use yuiui_support::slot_tree::NodeId;

use super::{Attributes, RcWidget};

#[derive(Debug)]
pub enum UnitOfWork<State, Message> {
    Append(NodeId, RcWidget<State, Message>, Rc<Attributes>),
    Insert(NodeId, RcWidget<State, Message>, Rc<Attributes>),
    Update(NodeId, RcWidget<State, Message>, Rc<Attributes>),
    Move(NodeId, NodeId),
    Remove(NodeId),
    RemoveChildren(NodeId),
}
