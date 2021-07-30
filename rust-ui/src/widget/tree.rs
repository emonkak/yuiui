use std::any::Any;
use std::sync::{Arc, Mutex};

use crate::bit_flags::BitFlags;
use crate::tree::{Link, NodeId, Tree};

use super::PolymophicWidget;
use super::element::{Children, Element, Key};

pub type WidgetTree<Handle> = Tree<WidgetPod<Handle>>;

pub type WidgetNode<Handle> = Link<WidgetPod<Handle>>;

#[derive(Debug)]
pub struct WidgetPod<Handle> {
    pub widget: Arc<dyn PolymophicWidget<Handle> + Send + Sync>,
    pub children: Children<Handle>,
    pub key: Option<Key>,
    pub state: Arc<Mutex<Box<dyn Any + Send + Sync>>>,
    pub deleted_children: Vec<NodeId>,
    pub flags: BitFlags<WidgetFlag>,
}

#[derive(Clone, Copy, Debug)]
pub enum WidgetFlag {
    None = 0b00,
    Dirty = 0b01,
    Fresh = 0b10,
}

impl<Handle> From<Element<Handle>> for WidgetPod<Handle> {
    fn from(element: Element<Handle>) -> Self {
        Self {
            state: Arc::new(Mutex::new(element.widget.initial_state())),
            widget: element.widget,
            children: element.children,
            key: element.key,
            deleted_children: Vec::new(),
            flags: WidgetFlag::Fresh.into(),
        }
    }
}

impl<Handle> Clone for WidgetPod<Handle> {
    fn clone(&self) -> Self {
        Self {
            widget: Arc::clone(&self.widget),
            children: Arc::clone(&self.children),
            key: self.key,
            state: Arc::clone(&self.state),
            deleted_children: self.deleted_children.clone(),
            flags: self.flags,
        }
    }
}

impl Into<usize> for WidgetFlag {
    fn into(self) -> usize {
        self as usize
    }
}
