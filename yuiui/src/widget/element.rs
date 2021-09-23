use super::*;
use std::array;
use std::rc::Rc;

pub type Key = usize;

#[derive(Clone, Debug)]
pub enum Element {
    WidgetElement(WidgetElement),
    ComponentElement(ComponentElement),
}

impl Element {
    pub fn new(
        node: ElementNode,
        children: Vec<Element>,
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

    pub fn create<const N: usize>(node: impl Into<ElementNode>, child_nodes: [Child; N]) -> Self {
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
pub struct WidgetElement {
    pub widget: BoxedWidget,
    pub children: Vec<Element>,
    pub attributes: Rc<Attributes>,
    pub key: Option<Key>,
}

#[derive(Clone, Debug)]
pub struct ComponentElement {
    pub component: BoxedComponent,
    pub children: Vec<Element>,
    pub attributes: Rc<Attributes>,
    pub key: Option<Key>,
}

#[derive(Clone, Debug)]
pub enum ElementNode {
    Widget(BoxedWidget),
    Component(BoxedComponent),
}

impl ElementNode {
    pub fn as_any(&self) -> &dyn Any {
        match self {
            Self::Widget(widget) => widget.as_any(),
            Self::Component(component) => component.as_any(),
        }
    }
}

impl From<BoxedWidget> for ElementNode {
    fn from(widget: BoxedWidget) -> Self {
        Self::Widget(widget)
    }
}

impl From<BoxedComponent> for ElementNode {
    fn from(component: BoxedComponent) -> Self {
        Self::Component(component)
    }
}

#[derive(Debug)]
pub enum Child {
    Multiple(Vec<Element>),
    Single(Element),
    Attribute(Box<dyn AnyValue>),
    Key(usize),
    None,
}

impl From<Vec<Element>> for Child {
    fn from(elements: Vec<Element>) -> Self {
        Child::Multiple(elements)
    }
}

impl From<Option<Element>> for Child {
    fn from(element: Option<Element>) -> Self {
        match element {
            Some(element) => Child::Single(element),
            None => Child::None,
        }
    }
}

impl From<Element> for Child {
    fn from(element: Element) -> Self {
        Child::Single(element)
    }
}

impl<T: 'static + Into<ElementNode>> From<T> for Child {
    fn from(node: T) -> Self {
        let element = Element::new(node.into(), vec![], Rc::new(Attributes::new()), None);
        Child::Single(element)
    }
}

pub fn attribute<T: 'static + AnyValue>(value: T) -> Child {
    Child::Attribute(Box::new(value))
}

pub fn key(key: Key) -> Child {
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
