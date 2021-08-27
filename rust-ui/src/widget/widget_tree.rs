use std::sync::{Arc, Mutex};

use crate::support::tree::{Link, NodeId, Tree};

use super::element::{Children, Element, Key};
use super::null::Null;
use super::state::State;
use super::widget::{PolymophicWidget, Widget};

pub type WidgetId = NodeId;

pub type WidgetTree<Renderer> = Tree<WidgetPod<Renderer>>;

pub type WidgetNode<Renderer> = Link<WidgetPod<Renderer>>;

#[derive(Debug)]
pub struct WidgetPod<Renderer> {
    pub widget: Arc<dyn PolymophicWidget<Renderer>>,
    pub children: Children<Renderer>,
    pub key: Option<Key>,
    pub state: Arc<State>,
}

#[derive(Debug)]
pub enum WidgetTreePatch<Renderer> {
    Append(WidgetId, WidgetPod<Renderer>),
    Insert(WidgetId, WidgetPod<Renderer>),
    Update(WidgetId, Element<Renderer>),
    Placement(WidgetId, WidgetId),
    Remove(WidgetId),
}

impl<Renderer> WidgetPod<Renderer> {
    #[inline]
    pub fn new<Widget>(widget: Widget, children: impl Into<Children<Renderer>>) -> Self
    where
        Widget: self::Widget<Renderer> + Send + Sync + 'static,
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
    pub fn should_update(&self, element: &Element<Renderer>) -> bool {
        self.widget.should_update(
            &*element.widget,
            &self.children,
            &element.children,
            self.state.clone(),
        )
    }

    #[inline]
    pub fn update(&mut self, element: Element<Renderer>) {
        self.widget = element.widget;
        self.children = element.children;
        self.key = element.key;
    }
}

impl<Renderer> From<Element<Renderer>> for WidgetPod<Renderer> {
    #[inline]
    fn from(element: Element<Renderer>) -> Self {
        Self {
            state: Arc::new(Mutex::new(element.widget.initial_state())),
            widget: element.widget,
            children: element.children,
            key: element.key,
        }
    }
}

impl<Renderer> Clone for WidgetPod<Renderer> {
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

pub fn create_widget_tree<Renderer>() -> (WidgetTree<Renderer>, WidgetId) {
    let mut tree = Tree::new();
    let root_id = tree.attach(WidgetPod::new(Null, Vec::new()));
    (tree, root_id)
}
