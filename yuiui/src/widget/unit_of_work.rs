use super::WidgetElement;
use yuiui_support::slot_tree::NodeId;

#[derive(Debug)]
pub enum UnitOfWork<State, Message> {
    Append(NodeId, WidgetElement<State, Message>),
    Insert(NodeId, WidgetElement<State, Message>),
    Update(NodeId, WidgetElement<State, Message>),
    UpdateAndMove(NodeId, NodeId, WidgetElement<State, Message>),
    Remove(NodeId),
    RemoveChildren(NodeId),
}
