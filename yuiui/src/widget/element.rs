use super::*;
use std::array;
use std::rc::Rc;

pub type Children<Message> = Rc<Vec<Element<Message>>>;

pub type Key = usize;

#[derive(Debug)]
pub enum Element<Message> {
    WidgetElement(WidgetElement<Message>),
    ComponentElement(ComponentElement<Message>),
}

impl<Message> Clone for Element<Message> {
    fn clone(&self) -> Self {
        match self {
            Self::WidgetElement(element) => Self::WidgetElement(element.clone()),
            Self::ComponentElement(element) => Self::ComponentElement(element.clone()),
        }
    }
}

impl<Message> Element<Message> {
    pub fn new(
        node: ElementInstance<Message>,
        attributes: Rc<Attributes>,
        key: Option<Key>,
        children: Children<Message>,
    ) -> Self {
        match node {
            ElementInstance::Widget(widget) => Self::WidgetElement(WidgetElement {
                widget,
                attributes,
                key,
                children,
            }),
            ElementInstance::Component(component) => Self::ComponentElement(ComponentElement {
                component,
                attributes,
                key,
                children,
            }),
        }
    }

    pub fn create<const N: usize>(
        node: impl Into<ElementInstance<Message>>,
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

        Self::new(node.into(), Rc::new(attributes), key, Rc::new(children))
    }
}

#[derive(Debug)]
pub struct WidgetElement<Message> {
    pub widget: RcWidget<Message>,
    pub children: Children<Message>,
    pub attributes: Rc<Attributes>,
    pub key: Option<Key>,
}

impl<Message> Clone for WidgetElement<Message> {
    fn clone(&self) -> Self {
        Self {
            widget: self.widget.clone(),
            attributes: self.attributes.clone(),
            key: self.key.clone(),
            children: self.children.clone(),
        }
    }
}

#[derive(Debug)]
pub struct ComponentElement<Message> {
    pub component: RcComponent<Message>,
    pub children: Children<Message>,
    pub attributes: Rc<Attributes>,
    pub key: Option<Key>,
}

impl<Message> Clone for ComponentElement<Message> {
    fn clone(&self) -> Self {
        Self {
            component: self.component.clone(),
            children: self.children.clone(),
            attributes: self.attributes.clone(),
            key: self.key.clone(),
        }
    }
}

#[derive(Clone, Debug)]
pub enum ElementInstance<Message> {
    Widget(RcWidget<Message>),
    Component(RcComponent<Message>),
}

impl<Message> From<RcWidget<Message>> for ElementInstance<Message> {
    fn from(widget: RcWidget<Message>) -> Self {
        Self::Widget(widget)
    }
}

impl<Message> From<RcComponent<Message>> for ElementInstance<Message> {
    fn from(component: RcComponent<Message>) -> Self {
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

impl<T: 'static + Into<ElementInstance<Message>>, Message> From<T> for Child<Message> {
    fn from(node: T) -> Self {
        let element = Element::new(
            node.into(),
            Rc::new(Attributes::new()),
            None,
            Rc::new(Vec::new()),
        );
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
