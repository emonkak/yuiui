use std::any::Any;
use std::sync::{Arc, Mutex};

use crate::tree::{Link, NodeId, Tree};

use super::element::{BoxedWidget, Children, Element, Key};
use super::Widget;

pub type WidgetTree<Painter> = Tree<WidgetPod<Painter>>;

pub type WidgetNode<Painter> = Link<WidgetPod<Painter>>;

#[derive(Debug)]
pub struct WidgetPod<Painter> {
    pub widget: BoxedWidget<Painter>,
    pub children: Children<Painter>,
    pub key: Option<Key>,
    pub state: Arc<Mutex<Box<dyn Any + Send + Sync>>>,
}

#[derive(Debug)]
pub enum Patch<Painter> {
    Append(NodeId, WidgetPod<Painter>),
    Insert(NodeId, WidgetPod<Painter>),
    Update(NodeId, Element<Painter>),
    Placement(NodeId, NodeId),
    Remove(NodeId),
}

impl<Painter> WidgetPod<Painter> {
    #[inline]
    pub fn new<Widget>(widget: Widget, children: impl Into<Children<Painter>>) -> Self
    where
        Widget: self::Widget<Painter> + Send + Sync + 'static,
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
    pub fn should_update(&self, element: &Element<Painter>) -> bool {
        self.widget.should_update(
            &*element.widget,
            &self.children,
            &element.children,
            &**self.state.lock().unwrap(),
        )
    }

    #[inline]
    pub fn update(&mut self, element: Element<Painter>) {
        self.widget = element.widget;
        self.children = element.children;
        self.key = element.key;
    }
}

impl<Painter> From<Element<Painter>> for WidgetPod<Painter> {
    #[inline]
    fn from(element: Element<Painter>) -> Self {
        Self {
            state: Arc::new(Mutex::new(element.widget.initial_state())),
            widget: element.widget,
            children: element.children,
            key: element.key,
        }
    }
}

impl<Painter> Clone for WidgetPod<Painter> {
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
