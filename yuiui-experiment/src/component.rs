use crate::element::{component, ComponentElement, Element};
use crate::hlist::{HCons, HList, HNil};

pub trait Component: Sized + 'static {
    type Element: Element;

    type State: Default;

    fn render(&self, state: &Self::State) -> Self::Element;

    fn should_update(&self, _other: &Self) -> bool {
        true
    }

    fn el(self) -> ComponentElement<Self> {
        component(self)
    }
}

#[derive(Debug)]
pub struct ComponentNode<C: Component> {
    pub component: C,
    pub state: C::State,
}

impl<C: Component> ComponentNode<C> {
    pub fn new(component: C) -> Self {
        Self {
            component,
            state: Default::default(),
        }
    }

    pub fn render(&self) -> C::Element {
        self.component.render(&self.state)
    }
}

pub trait ComponentStack: HList {
    fn mount(&mut self);

    fn unmount(&mut self);
}

impl<C: Component, CS: ComponentStack> ComponentStack for HCons<ComponentNode<C>, CS> {
    fn mount(&mut self) {
        self.1.mount();
    }

    fn unmount(&mut self) {
        self.0.state = Default::default();
        self.1.unmount();
    }
}

impl ComponentStack for HNil {
    fn mount(&mut self) {}

    fn unmount(&mut self) {}
}
