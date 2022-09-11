use crate::id::{Depth, Id, IdCounter, IdPath, IdPathBuf, StateTree};

#[derive(Debug)]
pub struct RenderContext {
    id_path: IdPathBuf,
    id_counter: IdCounter,
}

impl RenderContext {
    pub fn new() -> Self {
        Self {
            id_path: IdPathBuf::new(),
            id_counter: IdCounter::new(),
        }
    }

    pub(crate) fn begin_id(&mut self, id: Id) {
        self.id_path.push(id);
    }

    pub(crate) fn end_id(&mut self) {
        let old_id = self.id_path.pop();
        assert!(old_id.is_some());
    }

    pub(crate) fn with_id<F: FnOnce(Id, &mut Self) -> T, T>(&mut self, f: F) -> T {
        let id = self.id_counter.next();
        self.id_path.push(id);
        let result = f(id, self);
        self.id_path.pop();
        result
    }

    pub fn id_path(&self) -> &IdPath {
        &self.id_path
    }
}

#[derive(Debug, Clone)]
pub struct MessageContext<T> {
    id_path: IdPathBuf,
    state_tree: StateTree,
    messages: Vec<(T, StateTree)>,
}

impl<T> MessageContext<T> {
    pub fn new() -> Self {
        Self {
            id_path: IdPathBuf::new(),
            state_tree: StateTree::new(),
            messages: Vec::new(),
        }
    }

    pub(crate) fn new_sub_context<U>(
        &self,
        subscribers: Vec<(IdPathBuf, Depth)>,
    ) -> MessageContext<U> {
        let state_tree = self.state_tree.new_subtree(subscribers);
        MessageContext {
            id_path: self.id_path.clone(),
            state_tree,
            messages: Vec::new(),
        }
    }

    pub(crate) fn merge_sub_context<U, F: Fn(U) -> T>(
        &mut self,
        sub_context: MessageContext<U>,
        f: &F,
    ) {
        assert!(sub_context.id_path.starts_with(&self.id_path));
        let new_messages = sub_context
            .messages
            .into_iter()
            .map(|(message, state_tree)| (f(message), state_tree));
        self.messages.extend(new_messages);
    }

    pub(crate) fn begin_id(&mut self, id: Id) {
        self.id_path.push(id);
    }

    pub(crate) fn end_id(&mut self) {
        self.id_path.pop();
    }

    pub fn id_path(&self) -> &IdPath {
        &self.id_path
    }

    pub fn push_message(&mut self, message: T) {
        self.messages.push((message, self.state_tree.clone()));
    }

    pub fn into_messages(self) -> Vec<(T, StateTree)> {
        self.messages
    }
}
