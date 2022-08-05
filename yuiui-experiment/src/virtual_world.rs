use crate::context::Context;
use crate::element::Element;
use crate::real_world::RealWorld;
use crate::view::ViewPod;

pub struct VirtualWorld<E: Element> {
    view_pod: ViewPod<E::View, E::Components>,
    context: Context,
}

impl<E: Element> VirtualWorld<E> {
    pub fn new(element: E) -> Self {
        let mut context = Context::new(E::depth());
        let view_pod = element.build(&mut context);
        Self { view_pod, context }
    }

    pub fn realize(&self) -> RealWorld<E> {
        let widget_pod = E::compile(&self.view_pod);
        RealWorld::new(widget_pod)
    }

    pub fn view_pod(&self) -> &ViewPod<E::View, E::Components> {
        &self.view_pod
    }
}
