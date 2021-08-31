use std::marker::PhantomData;
use std::sync::Arc;

use crate::support::tree::{Link, NodeId, Tree};

use super::null::Null;
use super::widget::{PolyWidget, Proxy, Widget, WidgetSeal};

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
        W: Widget<R> + 'static,
        R: 'static,
    {
        let proxy = Proxy {
            widget,
            renderer_type: PhantomData,
            state_type: PhantomData,
            message_type: PhantomData,
        };
        Self {
            widget: Arc::new(proxy),
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

pub trait IntoElement<Renderer> {
    fn into_element(self) -> Element<Renderer>;
}

impl<W, R> IntoElement<R> for W
where
    W: Widget<R> + WidgetSeal + 'static,
    R: 'static,
{
    fn into_element(self) -> Element<R> {
        Element::new(self, None)
    }
}

impl<W, R> IntoElement<R> for WithKey<W>
where
    W: Widget<R> + 'static,
    R: 'static,
{
    fn into_element(self) -> Element<R> {
        Element::new(self.widget, Some(self.key))
    }
}

impl<R> From<Element<R>> for Children<R> {
    fn from(element: Element<R>) -> Self {
        vec![element]
    }
}

pub fn create_element_tree<R: 'static>() -> (ElementTree<R>, ElementId) {
    let mut tree = Tree::new();
    let root_id = tree.attach(Element::new(
        Null {
            children: Vec::new(),
        },
        None,
    ));
    (tree, root_id)
}
