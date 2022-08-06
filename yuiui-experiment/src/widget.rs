use std::fmt;

use crate::context::{Context, Id};
use crate::hlist::{HCons, HList};
use crate::sequence::WidgetNodeSeq;

pub trait Widget: 'static {
    type Children: WidgetNodeSeq;
}

pub struct WidgetNode<W: Widget, CS: HList> {
    pub id: Id,
    pub widget: W,
    pub children: W::Children,
    pub components: CS,
}

impl<W: Widget, CS: HList> WidgetNode<W, CS> {
    pub fn scope(&mut self) -> WidgetNodeScope<W, CS> {
        WidgetNodeScope {
            id: self.id,
            widget: &mut self.widget,
            children: &mut self.children,
            components: &mut self.components,
        }
    }

    pub fn commit(&mut self, context: &mut Context) {
        context.push(self.id);
        self.children.commit(context);
        context.pop();
    }

    pub fn invalidate(&mut self, _context: &mut Context) {}
}

impl<W, CS> fmt::Debug for WidgetNode<W, CS>
where
    W: Widget + fmt::Debug,
    W::Children: fmt::Debug,
    CS: HList + fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WidgetNode")
            .field("id", &self.id)
            .field("widget", &self.widget)
            .field("children", &self.children)
            .field("components", &self.components)
            .finish()
    }
}

pub struct WidgetNodeScope<'a, W: Widget, CS: HList> {
    pub id: Id,
    pub widget: &'a mut W,
    pub children: &'a mut W::Children,
    pub components: &'a mut CS,
}

impl<'a, W: Widget, C, CS: HList> WidgetNodeScope<'a, W, HCons<C, CS>> {
    pub fn destruct_components(self) -> (&'a mut C, WidgetNodeScope<'a, W, CS>) {
        let HCons(head_component, tail_components) = self.components;
        let node = WidgetNodeScope {
            id: self.id,
            widget: self.widget,
            children: self.children,
            components: tail_components,
        };
        (head_component, node)
    }
}
