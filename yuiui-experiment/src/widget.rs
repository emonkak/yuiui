use std::fmt;

use crate::context::{Context, Id};
use crate::sequence::{CommitMode, WidgetNodeSeq};
use crate::view::View;

pub trait Widget<S> {
    type Children: WidgetNodeSeq<S>;
}

pub struct WidgetNode<V: View<S>, CS, S> {
    pub id: Id,
    pub widget: V::Widget,
    pub pending_view: Option<V>,
    pub children: <V::Widget as Widget<S>>::Children,
    pub components: CS,
}

impl<V: View<S>, CS, S> WidgetNode<V, CS, S> {
    pub fn scope(&mut self) -> WidgetNodeScope<V, CS, S> {
        WidgetNodeScope {
            id: self.id,
            pending_view: &mut self.pending_view,
            children: &mut self.children,
            components: &mut self.components,
        }
    }

    pub fn commit(&mut self, mode: CommitMode, state: &S, context: &mut Context) {
        context.push(self.id);
        if let Some(view) = self.pending_view.take() {
            view.rebuild(&self.children, &mut self.widget, state);
        }
        self.children.commit(mode, state, context);
        context.pop();
    }
}

impl<V, CS, S> fmt::Debug for WidgetNode<V, CS, S>
where
    V: View<S> + fmt::Debug,
    V::Widget: fmt::Debug,
    <V::Widget as Widget<S>>::Children: fmt::Debug,
    CS: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WidgetNode")
            .field("id", &self.id)
            .field("widget", &self.widget)
            .field("pending_view", &self.pending_view)
            .field("children", &self.children)
            .field("components", &self.components)
            .finish()
    }
}

pub struct WidgetNodeScope<'a, V: View<S>, CS, S> {
    pub id: Id,
    pub pending_view: &'a mut Option<V>,
    pub children: &'a mut <V::Widget as Widget<S>>::Children,
    pub components: &'a mut CS,
}
