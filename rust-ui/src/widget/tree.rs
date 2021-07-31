use std::any::Any;
use std::sync::{Arc, Mutex};

use crate::tree::{Link, NodeId, Tree};

use super::element::{BoxedWidget, Children, Element, Key};
use super::Widget;

pub type WidgetTree<Handle> = Tree<WidgetPod<Handle>>;

pub type WidgetNode<Handle> = Link<WidgetPod<Handle>>;

#[derive(Debug)]
pub struct WidgetPod<Handle> {
    pub widget: BoxedWidget<Handle>,
    pub children: Children<Handle>,
    pub key: Option<Key>,
    pub state: Arc<Mutex<Box<dyn Any + Send + Sync>>>,
}

#[derive(Debug)]
pub enum Patch<Handle> {
    Append(NodeId, WidgetPod<Handle>),
    Insert(NodeId, WidgetPod<Handle>),
    Update(NodeId, Element<Handle>),
    Placement(NodeId, NodeId),
    Remove(NodeId),
}

impl<Handle> WidgetPod<Handle> {
    #[inline]
    pub fn new<Widget>(widget: Widget, children: impl Into<Children<Handle>>) -> Self
    where
        Widget: self::Widget<Handle> + Send + Sync + 'static,
        Widget::State: 'static,
    {
        Self {
            state: Arc::new(Mutex::new(Box::new(Widget::State::default()))),
            widget: Arc::new(widget),
            children: children.into(),
            key: None,
        }
    }

    #[inline]
    pub fn should_update(&self, element: &Element<Handle>) -> bool {
        self.widget.should_update(
            &*element.widget,
            &self.children,
            &element.children,
            &**self.state.lock().unwrap(),
        )
    }

    #[inline]
    pub fn update(&mut self, element: Element<Handle>) {
        self.widget = element.widget;
        self.children = element.children;
        self.key = element.key;
    }
}

impl<Handle> From<Element<Handle>> for WidgetPod<Handle> {
    #[inline]
    fn from(element: Element<Handle>) -> Self {
        Self {
            state: Arc::new(Mutex::new(element.widget.initial_state())),
            widget: element.widget,
            children: element.children,
            key: element.key,
        }
    }
}

impl<Handle> Clone for WidgetPod<Handle> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            widget: Arc::clone(&self.widget),
            children: Arc::clone(&self.children),
            key: self.key,
            state: Arc::clone(&self.state),
        }
    }
}
