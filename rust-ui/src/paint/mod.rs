pub mod tree;

use std::sync::Arc;

use crate::event::{EventHandler, EventManager, HandlerId};
use crate::geometrics::{Rectangle};

pub struct PaintContext<'a, Handle> {
    event_manager: &'a mut EventManager<Handle>,
    painter: &'a mut dyn Painter<Handle>,
}

#[derive(Debug, Eq, PartialEq)]
pub enum PaintHint {
    Always,
    Once,
}

#[derive(Debug)]
pub enum Lifecycle<Widget, Children> {
    OnMount(Children),
    OnUpdate(Widget, Children, Children),
    OnUnmount(Children),
}

pub trait Painter<Handle> {
    fn handle(&self) -> &Handle;

    fn fill_rectangle(&mut self, color: u32, rectangle: &Rectangle);

    fn commit(&mut self, rectangle: &Rectangle);
}

impl<'a, Handle> PaintContext<'a, Handle> {
    pub fn add_handler(
        &mut self,
        handler: Arc<dyn EventHandler<Handle> + Send + Sync>,
    ) -> HandlerId {
        self.event_manager.add(handler)
    }

    pub fn remove_handler(
        &mut self,
        handler_id: HandlerId,
    ) -> Arc<dyn EventHandler<Handle> + Send + Sync> {
        self.event_manager.remove(handler_id)
    }
}

impl<'a, Handle> Painter<Handle> for PaintContext<'a, Handle> {
    #[inline]
    fn handle(&self) -> &Handle {
        self.painter.handle()
    }

    #[inline]
    fn fill_rectangle(&mut self, color: u32, rectangle: &Rectangle) {
        self.painter.fill_rectangle(color, rectangle)
    }

    #[inline]
    fn commit(&mut self, rectangle: &Rectangle) {
        self.painter.commit(rectangle)
    }
}

impl<Widget, Children> Lifecycle<Widget, Children> {
    pub fn map<F, NewWidget>(self, f: F) -> Lifecycle<NewWidget, Children>
    where
        F: Fn(Widget) -> NewWidget,
    {
        match self {
            Lifecycle::OnMount(children) => Lifecycle::OnMount(children),
            Lifecycle::OnUpdate(widget, new_children, old_children) => {
                Lifecycle::OnUpdate(f(widget), new_children, old_children)
            }
            Lifecycle::OnUnmount(children) => Lifecycle::OnUnmount(children),
        }
    }

    pub fn without_params(&self) -> Lifecycle<(), ()> {
        match self {
            Lifecycle::OnMount(_) => Lifecycle::OnMount(()),
            Lifecycle::OnUpdate(_, _, _) => Lifecycle::OnUpdate((), (), ()),
            Lifecycle::OnUnmount(_) => Lifecycle::OnUnmount(()),
        }
    }
}
