use std::any::{self, Any};
use std::fmt;

use crate::element_seq::ElementSeq;
use crate::widget::{AnyWidget, Widget};

pub trait View: 'static + AnyView {
    type Widget: Widget;

    type Children: ElementSeq<Widgets = <Self::Widget as Widget>::Children>;

    fn build(&self, children: &<Self::Children as ElementSeq>::Views) -> Self::Widget;

    fn rebuild(
        &self,
        children: &<Self::Children as ElementSeq>::Views,
        widget: &mut Self::Widget,
    ) -> bool {
        *widget = View::build(self, children);
        true
    }
}

pub trait AnyView {
    fn build(&self, children: &Box<dyn Any>) -> Box<dyn AnyWidget>;

    fn rebuild(&self, children: &Box<dyn Any>, widget: &mut Box<dyn AnyWidget>) -> bool;

    fn name(&self) -> &'static str;

    fn as_any(&self) -> &dyn Any;

    fn as_any_mut(&mut self) -> &mut dyn Any;
}

impl<T: View> AnyView for T {
    fn build(&self, children: &Box<dyn Any>) -> Box<dyn AnyWidget> {
        Box::new(View::build(self, *children.downcast_ref().unwrap()))
    }

    fn rebuild(&self, children: &Box<dyn Any>, widget: &mut Box<dyn AnyWidget>) -> bool {
        View::rebuild(
            self,
            children.downcast_ref().unwrap(),
            widget.as_any_mut().downcast_mut().unwrap(),
        )
    }

    fn name(&self) -> &'static str {
        any::type_name::<T>()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

pub struct ViewPod<V: View, C> {
    pub(crate) view: V,
    pub(crate) children: <V::Children as ElementSeq>::Views,
    pub(crate) components: C,
}

impl<V, C> fmt::Debug for ViewPod<V, C>
where
    V: View + fmt::Debug,
    <V::Children as ElementSeq>::Views: fmt::Debug,
    C: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ViewPod")
            .field("view", &self.view)
            .field("children", &self.children)
            .field("components", &self.components)
            .finish()
    }
}

pub trait ViewInspector: Sized {
    type Id: Copy;

    fn push<V: View, C>(&mut self, origin: Self::Id, view_pod: &ViewPod<V, C>) -> Self::Id;
}
