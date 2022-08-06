use std::fmt;

use crate::context::{Context, Id};
use crate::hlist::{HCons, HList};
use crate::sequence::{ElementSeq, ViewNodeSeq};
use crate::view::View;

pub struct ViewNode<V: View, CS: HList> {
    pub id: Id,
    pub view: V,
    pub widget: V::Widget,
    pub children: <V::Children as ElementSeq>::Nodes,
    pub components: CS,
}

impl<V: View, CS: HList> ViewNode<V, CS> {
    pub fn scope(&mut self) -> ViewNodeScope<V, CS> {
        ViewNodeScope {
            id: self.id,
            view: &mut self.view,
            widget: &mut self.widget,
            children: &mut self.children,
            components: &mut self.components,
        }
    }

    pub fn commit(&mut self, context: &mut Context) {
        context.push(self.id);
        ViewNodeSeq::commit(&mut self.children, context);
        context.pop();
    }

    pub fn invalidate(&mut self, _context: &mut Context) {}
}

impl<V, CS> fmt::Debug for ViewNode<V, CS>
where
    V: View + fmt::Debug,
    V::Widget: fmt::Debug,
    <V::Children as ElementSeq>::Nodes: fmt::Debug,
    CS: HList + fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ViewNode")
            .field("id", &self.id)
            .field("view", &self.view)
            .field("widget", &self.view)
            .field("children", &self.children)
            .field("components", &self.components)
            .finish()
    }
}

pub struct ViewNodeScope<'a, V: View, CS: HList> {
    pub id: Id,
    pub view: &'a mut V,
    pub widget: &'a mut V::Widget,
    pub children: &'a mut <V::Children as ElementSeq>::Nodes,
    pub components: &'a mut CS,
}

impl<'a, V: View, C, CS: HList> ViewNodeScope<'a, V, HCons<C, CS>> {
    pub fn destruct_components(self) -> (&'a mut C, ViewNodeScope<'a, V, CS>) {
        let HCons(head_component, tail_components) = self.components;
        let node = ViewNodeScope {
            id: self.id,
            view: self.view,
            widget: self.widget,
            children: self.children,
            components: tail_components,
        };
        (head_component, node)
    }
}
