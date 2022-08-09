use std::fmt;

use crate::component::ComponentStack;
use crate::context::{BuildContext, Id};
use crate::sequence::{CommitMode, WidgetNodeSeq};
use crate::state::State;
use crate::view::View;

pub trait Widget<S: State> {
    type Children: WidgetNodeSeq<S>;
}

pub struct WidgetNode<V: View<S>, CS, S: State> {
    pub id: Id,
    pub status: Option<WidgetStatus<V, V::Widget>>,
    pub children: <V::Widget as Widget<S>>::Children,
    pub components: CS,
}

impl<V, CS, S> WidgetNode<V, CS, S>
where
    V: View<S>,
    CS: ComponentStack<S>,
    S: State,
{
    pub fn scope(&mut self) -> WidgetNodeScope<V, CS, S> {
        WidgetNodeScope {
            id: self.id,
            status: &mut self.status,
            children: &mut self.children,
            components: &mut self.components,
        }
    }

    pub fn commit(&mut self, mode: CommitMode, state: &S, context: &mut BuildContext<S>) {
        context.begin(self.id);
        context.begin_components();
        self.components.commit(mode, state, context);
        context.end_components();
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
        context.end();
    }
}

impl<V, CS, S> fmt::Debug for WidgetNode<V, CS, S>
where
    V: View<S> + fmt::Debug,
    V::Widget: fmt::Debug,
    <V::Widget as Widget<S>>::Children: fmt::Debug,
    CS: fmt::Debug,
    S: State,
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

pub struct WidgetNodeScope<'a, V: View<S>, CS, S: State> {
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
