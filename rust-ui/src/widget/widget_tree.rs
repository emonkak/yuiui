use std::sync::{Arc, RwLock};

use crate::support::tree::{Link, NodeId, Tree};

use super::effect::Effect;
use super::element::{Children, Element, Key};
use super::null::Null;
use super::widget::{PolymophicWidget, Widget, StateHolder};

pub type WidgetId = NodeId;

pub type WidgetTree<Renderer> = Tree<WidgetPod<Renderer>>;

pub type WidgetNode<Renderer> = Link<WidgetPod<Renderer>>;

#[derive(Debug)]
pub struct WidgetPod<Renderer> {
    pub widget: Arc<dyn PolymophicWidget<Renderer>>,
    pub children: Children<Renderer>,
    pub key: Option<Key>,
    pub state: StateHolder,
}

#[derive(Debug)]
pub enum WidgetPatch<Renderer> {
    Append(WidgetId, WidgetPod<Renderer>),
    Insert(WidgetId, WidgetPod<Renderer>),
    Update(WidgetId, Element<Renderer>),
    Move(WidgetId, WidgetId),
    Remove(WidgetId),
    Effect(WidgetId, Vec<Effect<Renderer>>),
}

impl<Renderer> WidgetPod<Renderer> {
    #[inline]
    pub fn new<Widget>(widget: Widget, children: impl Into<Children<Renderer>>) -> Self
    where
        Widget: self::Widget<Renderer> + Send + 'static,
        Widget::State: 'static,
    {
        Self {
            state: Arc::new(RwLock::new(Box::new(Widget::State::default()))),
            widget: Arc::new(widget),
            children: children.into(),
            key: None,
        }
    }

    #[inline]
    pub fn should_update(&self, element: &Element<Renderer>) -> bool {
        self.widget.should_update(
            &self.children,
            &self.state,
            &*element.widget,
            &element.children,
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
            state: Arc::new(RwLock::new(element.widget.initial_state())),
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
            widget: self.widget.clone(),
            children: self.children.clone(),
            key: self.key,
            state: self.state.clone(),
        }
    }
}

pub fn create_widget_tree<Renderer>() -> (WidgetTree<Renderer>, WidgetId) {
    let mut tree = Tree::new();
    let root_id = tree.attach(WidgetPod::new(Null, Vec::new()));
    (tree, root_id)
}
