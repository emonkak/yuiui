use crate::cancellation_token::CancellationToken;
use crate::command::Command;
use crate::element::Element;
use crate::id::{IdPath, IdStack, Level};
use crate::state::Atom;
use crate::view_node::ViewNode;

#[derive(Debug)]
pub struct RenderContext<'context, S> {
    pub(crate) id_stack: &'context mut IdStack,
    pub(crate) state: &'context S,
    pub(crate) level: Level,
}

impl<'context, S> RenderContext<'context, S> {
    #[inline]
    pub fn id_path(&self) -> &IdPath {
        self.id_stack.id_path()
    }

    #[inline]
    pub fn state(&self) -> &S {
        self.state
    }

    #[inline]
    pub fn use_atom<F, T>(&self, f: F) -> &T
    where
        F: FnOnce(&S) -> &Atom<T>,
    {
        let atom = f(self.state);
        atom.subscribe(self.id_stack.id_path(), self.level);
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
    pub(crate) commands: &'context mut Vec<(Command<M>, Option<CancellationToken>)>,
    pub(crate) entry_point: &'context E,
}

impl<'context, S, M, E> CommitContext<'context, S, M, E> {
    #[inline]
    pub fn id_path(&self) -> &IdPath {
        self.id_stack.id_path()
    }

    #[inline]
    pub fn state(&self) -> &S {
        self.state
    }

    #[inline]
    pub fn entry_point(&self) -> &E {
        self.entry_point
    }

    #[inline]
    pub fn dispatch(&mut self, message: M) {
        self.messages.push(message);
    }

    #[inline]
    pub fn spawn(&mut self, command: Command<M>, cancellation_token: Option<CancellationToken>) {
        self.commands.push((command, cancellation_token));
    }

    pub(crate) fn enter_sub_context<F, FS, FM, T, SS, SM>(
        &mut self,
        select_state: &FS,
        lift_message: &FM,
        f: F,
    ) -> T
    where
        F: FnOnce(CommitContext<SS, SM, E>) -> T,
        FS: Fn(&S) -> &SS,
        FM: Fn(SM) -> M + Clone + Send + 'static,
        SM: 'static,
    {
        let mut messages = Vec::new();
        let mut commands = Vec::new();
        let inner_context = CommitContext {
            id_stack: self.id_stack,
            state: select_state(&self.state),
            messages: &mut messages,
            commands: &mut commands,
            entry_point: self.entry_point,
        };
        let result = f(inner_context);
        self.messages.extend(messages.into_iter().map(lift_message));
        self.commands
            .extend(commands.into_iter().map(|(command, cancellation_token)| {
                (command.map(lift_message.clone()), cancellation_token)
            }));
        result
    }
}
