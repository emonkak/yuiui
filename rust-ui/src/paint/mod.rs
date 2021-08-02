pub mod layout;
pub mod tree;

use std::sync::Arc;

use crate::event::{EventHandler, EventManager, HandlerId};

pub struct PaintContext<'a, Painter> {
    event_manager: &'a mut EventManager<Painter>,
}

#[derive(Debug)]
pub enum PaintCycle<Widget, Children> {
    DidMount(Children),
    DidUpdate(Children, Widget, Children),
    DidUnmount(Children),
}

#[derive(Debug)]
pub enum PaintHint {
    Always,
    Once,
}

impl<'a, Painter> PaintContext<'a, Painter> {
    pub fn add_handler(&mut self, handler: Arc<dyn EventHandler<Painter>>) -> HandlerId {
        self.event_manager.add(handler)
    }

    pub fn remove_handler(&mut self, handler_id: HandlerId) -> Arc<dyn EventHandler<Painter>> {
        self.event_manager.remove(handler_id)
    }
}

impl<Widget, Children> PaintCycle<Widget, Children> {
    pub fn map<F, NewWidget>(self, f: F) -> PaintCycle<NewWidget, Children>
    where
        F: Fn(Widget) -> NewWidget,
    {
        match self {
            PaintCycle::DidMount(children) => PaintCycle::DidMount(children),
            PaintCycle::DidUpdate(children, new_widget, new_children) => {
                PaintCycle::DidUpdate(children, f(new_widget), new_children)
            }
            PaintCycle::DidUnmount(children) => PaintCycle::DidUnmount(children),
        }
    }

    pub fn without_params(&self) -> PaintCycle<(), ()> {
        match self {
            PaintCycle::DidMount(_) => PaintCycle::DidMount(()),
            PaintCycle::DidUpdate(_, _, _) => PaintCycle::DidUpdate((), (), ()),
            PaintCycle::DidUnmount(_) => PaintCycle::DidUnmount(()),
        }
    }
}
