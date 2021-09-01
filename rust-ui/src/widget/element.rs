use std::sync::Arc;

use crate::support::tree::{Link, NodeId, Tree};

use super::null::Null;
use super::state::State;
use super::widget::{PolyWidget, Widget, WidgetProxy, WidgetSeal};

pub type ElementTree<Renderer> = Tree<Element<Renderer>>;

pub type ElementNode<Renderer> = Link<Element<Renderer>>;

pub type ElementId = NodeId;

pub type Children<Renderer> = Vec<Element<Renderer>>;

pub type Key = usize;

#[derive(Debug)]
pub struct Element<R> {
    pub widget: Arc<PolyWidget<R>>,
    pub key: Option<Key>,
}

#[derive(Debug)]
pub struct WithKey<W> {
    pub widget: W,
    pub key: Key,
}

#[derive(Debug)]
pub enum Patch<R> {
    Append(ElementId, Element<R>),
    Insert(ElementId, Element<R>),
    Update(ElementId, Element<R>),
    Move(ElementId, ElementId),
    Remove(ElementId),
}

impl<R> Element<R> {
    pub fn new<W>(widget: W, key: Option<Key>) -> Self
    where
        W: 'static + Widget<R>,
        W::State: Sized,
        W::Message: Sized,
        R: 'static,
    {
        Self {
            widget: Arc::new(WidgetProxy::new(widget)),
            key,
        }
    }
}

impl<R> Clone for Element<R> {
    fn clone(&self) -> Self {
        Self {
            widget: Arc::clone(&self.widget),
            key: self.key,
        }
    }
}

impl<R> From<Element<R>> for Children<R> {
    fn from(element: Element<R>) -> Self {
        vec![element]
    }
}

pub trait IntoElement<Renderer> {
    fn into_element(self) -> Element<Renderer>;
}

impl<W, R> IntoElement<R> for W
where
    W: 'static + Widget<R> + WidgetSeal,
    W::State: Sized,
    W::Message: Sized,
    R: 'static,
{
    fn into_element(self) -> Element<R> {
        Element::new(self, None)
    }
}

impl<W, R> IntoElement<R> for WithKey<W>
where
    W: 'static + Widget<R>,
    W::State: Sized,
    W::Message: Sized,
    R: 'static,
{
    fn into_element(self) -> Element<R> {
        Element::new(self.widget, Some(self.key))
    }
}

pub fn create_element_tree<R: 'static>() -> (ElementTree<R>, ElementId, State<R>) {
    let mut tree = Tree::new();
    let root = Null {
        children: Vec::new(),
    };
    let initial_state = root.initial_state().into();
    let root_id = tree.attach(root.into_element());
    (tree, root_id, initial_state)
}
