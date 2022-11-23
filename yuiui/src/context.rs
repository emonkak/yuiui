use crate::id::{IdPath, IdStack};
use crate::state::Atom;

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
        atom.subscribe(self.id_stack.id_path(), self.id_stack.depth());
        atom.peek()
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
