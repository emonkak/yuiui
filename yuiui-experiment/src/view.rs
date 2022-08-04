use std::any::{self, Any};
use std::convert::Infallible;

use crate::element_seq::ElementSeq;
use crate::widget::{AnyWidget, Widget};
use crate::world::{Id, World};

pub trait View: 'static + AnyView {
    type Widget: Widget;

    type Children: ElementSeq;

    fn build(&self, children: &Self::Children) -> Self::Widget;

    fn rebuild(&self, children: &Self::Children, widget: &mut Self::Widget) -> bool {
        *widget = View::build(self, children);
        true
    }

    fn reconcile_children(
        &self,
        children: Self::Children,
        target: Id,
        world: &mut World,
    ) -> Option<Id> {
        children.reconcile(target, world)
    }
}

impl View for Infallible {
    type Widget = ();

    type Children = ();

    fn build(&self, _children: &Self::Children) -> Self::Widget {
        ()
    }
}

pub trait AnyView {
    fn build(&self, children: &Box<dyn Any>) -> Box<dyn AnyWidget>;

    fn rebuild(&self, children: &Box<dyn Any>, widget: &mut Box<dyn AnyWidget>) -> bool;

    fn reconcile_children(
        &self,
        children: Box<dyn Any>,
        target: Id,
        world: &mut World,
    ) -> Option<Id>;

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

    fn reconcile_children(
        &self,
        children: Box<dyn Any>,
        target: Id,
        world: &mut World,
    ) -> Option<Id> {
        View::reconcile_children(self, *children.downcast().unwrap(), target, world)
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
