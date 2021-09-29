use super::*;
use std::array;
use std::rc::Rc;

pub type Children<State, Message> = Rc<Vec<Element<State, Message>>>;

pub type Key = usize;

#[derive(Debug)]
pub enum Element<State, Message> {
    WidgetElement(WidgetElement<State, Message>),
    ComponentElement(ComponentElement<State, Message>),
}

impl<State, Message> Clone for Element<State, Message> {
    fn clone(&self) -> Self {
        match self {
            Self::WidgetElement(element) => Self::WidgetElement(element.clone()),
            Self::ComponentElement(element) => Self::ComponentElement(element.clone()),
        }
    }
}

impl<State, Message> Element<State, Message> {
    pub fn new(
        node: ElementInstance<State, Message>,
        attributes: Rc<Attributes>,
        key: Option<Key>,
        children: Children<State, Message>,
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
        node: impl Into<ElementInstance<State, Message>>,
        child_nodes: [Child<State, Message>; N],
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
pub struct WidgetElement<State, Message> {
    pub widget: RcWidget<State, Message>,
    pub children: Children<State, Message>,
    pub attributes: Rc<Attributes>,
    pub key: Option<Key>,
}

impl<State, Message> Clone for WidgetElement<State, Message> {
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
pub struct ComponentElement<State, Message> {
    pub component: RcComponent<State, Message>,
    pub children: Children<State, Message>,
    pub attributes: Rc<Attributes>,
    pub key: Option<Key>,
}

impl<State, Message> Clone for ComponentElement<State, Message> {
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
pub enum ElementInstance<State, Message> {
    Widget(RcWidget<State, Message>),
    Component(RcComponent<State, Message>),
}

impl<State, Message> From<RcWidget<State, Message>> for ElementInstance<State, Message> {
    fn from(widget: RcWidget<State, Message>) -> Self {
        Self::Widget(widget)
    }
}

impl<State, Message> From<RcComponent<State, Message>> for ElementInstance<State, Message> {
    fn from(component: RcComponent<State, Message>) -> Self {
        Self::Component(component)
    }
}

#[derive(Debug)]
pub enum Child<State, Message> {
    Multiple(Vec<Element<State, Message>>),
    Single(Element<State, Message>),
    Attribute(Box<dyn AnyValue>),
    Key(usize),
    None,
}

impl<State, Message> From<Vec<Element<State, Message>>> for Child<State, Message> {
    fn from(elements: Vec<Element<State, Message>>) -> Self {
        Child::Multiple(elements)
    }
}

impl<State, Message> From<Option<Element<State, Message>>> for Child<State, Message> {
    fn from(element: Option<Element<State, Message>>) -> Self {
        match element {
            Some(element) => Child::Single(element),
            None => Child::None,
        }
    }
}

impl<State, Message> From<Element<State, Message>> for Child<State, Message> {
    fn from(element: Element<State, Message>) -> Self {
        Child::Single(element)
    }
}

impl<State, Message, T> From<T> for Child<State, Message>
where
    T: 'static + Into<ElementInstance<State, Message>>,
{
    fn from(instance: T) -> Self {
        let element = Element::new(
            instance.into(),
            Rc::new(Attributes::new()),
            None,
            Rc::new(Vec::new()),
        );
        Child::Single(element)
    }
}

pub fn attribute<State, Message, Value>(value: Value) -> Child<State, Message>
where
    Value: 'static + AnyValue,
{
    Child::Attribute(Box::new(value))
}

pub fn key<State, Message>(key: Key) -> Child<State, Message> {
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
