use super::*;
use std::array;
use std::rc::Rc;

pub type Key = usize;

#[derive(Clone, Debug)]
pub enum Element<Message> {
    WidgetElement(WidgetElement<Message>),
    ComponentElement(ComponentElement<Message>),
}

impl<Message> Element<Message> {
    pub fn new(
        node: ElementNode<Message>,
        children: Vec<Element<Message>>,
        attributes: Rc<Attributes>,
        key: Option<Key>,
    ) -> Self {
        match node {
            ElementNode::Widget(widget) => Self::WidgetElement(WidgetElement {
                widget,
                attributes,
                children,
                key,
            }),
            ElementNode::Component(component) => Self::ComponentElement(ComponentElement {
                component,
                attributes,
                children,
                key,
            }),
        }
    }

    pub fn create<const N: usize>(
        node: impl Into<ElementNode<Message>>,
        child_nodes: [Child<Message>; N],
    ) -> Self {
        let mut attributes = Attributes::new();
        let mut children = Vec::new();
        let mut key = None;

        for child_node in array::IntoIter::new(child_nodes) {
            match child_node {
                Child::Multiple(elements) => children.extend(elements),
                Child::Single(element) => children.push(element),
                Child::Attribute(value) => attributes.add(value),
                Child::Key(value) => key = Some(value),
                Child::None => {}
            }
        }

        Self::new(node.into(), children, Rc::new(attributes), key)
    }
}

#[derive(Clone, Debug)]
pub struct WidgetElement<Message> {
    pub widget: BoxedWidget<Message>,
    pub children: Vec<Element<Message>>,
    pub attributes: Rc<Attributes>,
    pub key: Option<Key>,
}

#[derive(Clone, Debug)]
pub struct ComponentElement<Message> {
    pub component: BoxedComponent<Message>,
    pub children: Vec<Element<Message>>,
    pub attributes: Rc<Attributes>,
    pub key: Option<Key>,
}

#[derive(Clone, Debug)]
pub enum ElementNode<Message> {
    Widget(BoxedWidget<Message>),
    Component(BoxedComponent<Message>),
}

impl<Message> From<BoxedWidget<Message>> for ElementNode<Message> {
    fn from(widget: BoxedWidget<Message>) -> Self {
        Self::Widget(widget)
    }
}

impl<Message> From<BoxedComponent<Message>> for ElementNode<Message> {
    fn from(component: BoxedComponent<Message>) -> Self {
        Self::Component(component)
    }
}

#[derive(Debug)]
pub enum Child<Message> {
    Multiple(Vec<Element<Message>>),
    Single(Element<Message>),
    Attribute(Box<dyn AnyValue>),
    Key(usize),
    None,
}

impl<Message> From<Vec<Element<Message>>> for Child<Message> {
    fn from(elements: Vec<Element<Message>>) -> Self {
        Child::Multiple(elements)
    }
}

impl<Message> From<Option<Element<Message>>> for Child<Message> {
    fn from(element: Option<Element<Message>>) -> Self {
        match element {
            Some(element) => Child::Single(element),
            None => Child::None,
        }
    }
}

impl<Message> From<Element<Message>> for Child<Message> {
    fn from(element: Element<Message>) -> Self {
        Child::Single(element)
    }
}

impl<T: 'static + Into<ElementNode<Message>>, Message> From<T> for Child<Message> {
    fn from(node: T) -> Self {
        let element = Element::new(node.into(), vec![], Rc::new(Attributes::new()), None);
        Child::Single(element)
    }
}

pub fn attribute<T: 'static + AnyValue, Message>(value: T) -> Child<Message> {
    Child::Attribute(Box::new(value))
}

pub fn key<Message>(key: Key) -> Child<Message> {
    Child::Key(key)
}

#[macro_export]
macro_rules! element {
    ($expr:expr => [ $($content:tt)* ]) => {
        $crate::widget::Element::create($expr, __element_children!([] $($content)*))
    };
    ($expr:expr => $child:expr) => {
        element!($expr => { $child })
    };
    ($expr:expr) => {
        $crate::widget::Element::create($expr, [])
    };
}

#[macro_export]
macro_rules! __element_children {
    ([$($children:expr)*] $expr:expr => [ $($content:tt)* ] $($rest:tt)*) => {
        __element_children!([$($children)* $crate::widget::Child::Single($crate::widget::Element::create($expr, __element_children!([] $($content)*)))] $($rest)*)
    };
    ([$($children:expr)*] $expr:expr => $child:expr, $($rest:tt)*) => {
        __element_children!([$($children)*] $expr => [ $child ] $($rest)*)
    };
    ([$($children:expr)*] $expr:expr => $child:expr) => {
        __element_children!([$($children)*] $expr => [ $child ])
    };
    ([$($children:expr)*] $expr:expr, $($rest:tt)*) => {
        __element_children!([$($children)* $crate::widget::Child::from($expr)] $($rest)*)
    };
    ([$($children:expr)*] $expr:expr) => {
        __element_children!([$($children)* $crate::widget::Child::from($expr)])
    };
    ([$($children:expr)*]) => {
        [$($children),*]
    };
}
