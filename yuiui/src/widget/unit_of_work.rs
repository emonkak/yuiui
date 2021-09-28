use super::WidgetElement;
use yuiui_support::slot_tree::NodeId;

#[derive(Debug)]
pub enum UnitOfWork<Message> {
    Append(NodeId, WidgetElement<Message>),
    Insert(NodeId, WidgetElement<Message>),
    Update(NodeId, WidgetElement<Message>),
    UpdateAndMove(NodeId, NodeId, WidgetElement<Message>),
    Remove(NodeId),
    RemoveChildren(NodeId),
}
