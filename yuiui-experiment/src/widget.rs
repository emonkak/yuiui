use std::fmt;

use crate::context::{Context, Id};
use crate::sequence::{CommitMode, WidgetNodeSeq};
use crate::view::View;

pub trait Widget<S> {
    type Children: WidgetNodeSeq<S>;
}

pub struct WidgetNode<V: View<S>, CS, S> {
    pub id: Id,
    pub status: Option<WidgetStatus<V, V::Widget>>,
    pub children: <V::Widget as Widget<S>>::Children,
    pub components: CS,
}

impl<V: View<S>, CS, S> WidgetNode<V, CS, S> {
    pub fn scope(&mut self) -> WidgetNodeScope<V, CS, S> {
        WidgetNodeScope {
            id: self.id,
            status: &mut self.status,
            children: &mut self.children,
            components: &mut self.components,
        }
    }

    pub fn commit(&mut self, mode: CommitMode, state: &S, context: &mut Context) {
        context.push(self.id);
        self.status = match self.status.take().unwrap() {
            WidgetStatus::Prepared(widget) => WidgetStatus::Prepared(widget),
            WidgetStatus::Changed(mut widget, view) => {
                view.rebuild(&self.children, &mut widget, state);
                WidgetStatus::Prepared(widget)
            }
            WidgetStatus::Uninitialized(view) => {
                let widget = view.build(&self.children, state);
                WidgetStatus::Prepared(widget)
            }
        }
        .into();
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
            .field("status", &self.status)
            .field("children", &self.children)
            .field("components", &self.components)
            .finish()
    }
}

pub struct WidgetNodeScope<'a, V: View<S>, CS, S> {
    pub id: Id,
    pub status: &'a mut Option<WidgetStatus<V, V::Widget>>,
    pub children: &'a mut <V::Widget as Widget<S>>::Children,
    pub components: &'a mut CS,
}

#[derive(Debug)]
pub enum WidgetStatus<V, W> {
    Prepared(W),
    Changed(W, V),
    Uninitialized(V),
}
