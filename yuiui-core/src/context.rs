use crate::element::Element;
use crate::id::{IdPath, IdStack};
use crate::state::Atom;
use crate::view_node::ViewNode;

#[derive(Debug)]
pub struct RenderContext<'context, S> {
    pub(crate) id_stack: &'context mut IdStack,
    pub(crate) state: &'context S,
}

impl<'context, S> RenderContext<'context, S> {
    pub fn id_path(&self) -> &IdPath {
        self.id_stack.id_path()
    }

    pub fn use_atom<F, T>(&self, f: F) -> &T
    where
        F: FnOnce(&S) -> &Atom<T>,
    {
        let atom = f(self.state);
        atom.subscribe(self.id_stack.id_path(), self.id_stack.level());
        atom.get()
    }

    pub(crate) fn render_node<Element, M, E>(
        &mut self,
        element: Element,
    ) -> ViewNode<Element::View, Element::Components, S, M, E>
    where
        Element: self::Element<S, M, E>,
    {
        let id = self.id_stack.next();
        self.id_stack.push(id);
        let node = element.render(self);
        self.id_stack.pop();
        node
    }

    pub(crate) fn update_node<Element, M, E>(
        &mut self,
        element: Element,
        node: &mut ViewNode<Element::View, Element::Components, S, M, E>,
    ) -> bool
    where
        Element: self::Element<S, M, E>,
    {
        self.id_stack.push(node.id);
        let has_changed = element.update(&mut node.into(), self);
        self.id_stack.pop();
        has_changed
    }
}

#[derive(Debug)]
pub struct CommitContext<'context, S, M, E> {
    pub(crate) id_stack: &'context mut IdStack,
    pub(crate) state: &'context S,
    pub(crate) messages: &'context mut Vec<M>,
    pub(crate) entry_point: &'context E,
}

impl<'context, S, M, E> CommitContext<'context, S, M, E> {
    pub fn id_path(&self) -> &IdPath {
        self.id_stack.id_path()
    }

    pub fn state(&self) -> &S {
        self.state
    }

    pub fn entry_point(&self) -> &E {
        self.entry_point
    }

    pub fn push_message(&mut self, message: M) {
        self.messages.push(message);
    }
}
