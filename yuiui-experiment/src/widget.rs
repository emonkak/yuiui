use std::fmt;

use crate::context::{Context, Id};
use crate::sequence::WidgetNodeSeq;
use crate::view::View;

pub trait Widget: 'static {
    type Children: WidgetNodeSeq;
}

pub struct WidgetNode<V: View, CS> {
    pub id: Id,
    pub widget: V::Widget,
    pub pending_view: Option<V>,
    pub children: <V::Widget as Widget>::Children,
    pub components: CS,
}

impl<V: View, CS> WidgetNode<V, CS> {
    pub fn scope(&mut self) -> WidgetNodeScope<V, CS> {
        WidgetNodeScope {
            id: self.id,
            pending_view: &mut self.pending_view,
            children: &mut self.children,
            components: &mut self.components,
        }
    }

    pub fn commit(&mut self, mode: CommitMode, context: &mut Context) {
        context.push(self.id);
        if let Some(view) = self.pending_view.take() {
            view.rebuild(&self.children, &mut self.widget);
        }
        self.children.commit(mode, context);
        context.pop();
    }
}

impl<V, CS> fmt::Debug for WidgetNode<V, CS>
where
    V: View + fmt::Debug,
    V::Widget: fmt::Debug,
    <V::Widget as Widget>::Children: fmt::Debug,
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

pub struct WidgetNodeScope<'a, V: View, CS> {
    pub id: Id,
    pub pending_view: &'a mut Option<V>,
    pub children: &'a mut <V::Widget as Widget>::Children,
    pub components: &'a mut CS,
}

#[derive(Clone, Copy, Debug)]
pub enum CommitMode {
    Mount,
    Unmount,
    Update,
}

impl CommitMode {
    pub fn is_propagatable(&self) -> bool {
        match self {
            Self::Mount => true,
            Self::Unmount => true,
            Self::Update => false,
        }
    }

    pub fn is_unmount(&self) -> bool {
        match self {
            Self::Mount => false,
            Self::Unmount => true,
            Self::Update => false,
        }
    }
}
