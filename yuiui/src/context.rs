use crate::id::{Id, IdCounter, IdPath, IdPathBuf};

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
        debug_assert!(old_id.is_some());
    }

    pub(crate) fn next_id(&mut self) -> Id {
        self.id_counter.next()
    }

    pub fn id_path(&self) -> &IdPath {
        &self.id_path
    }
}

#[derive(Debug)]
pub struct MessageContext<T> {
    id_path: IdPathBuf,
    messages: Vec<T>,
}

impl<T> MessageContext<T> {
    pub fn new() -> Self {
        Self {
            id_path: IdPathBuf::new(),
            messages: Vec::new(),
        }
    }

    pub(crate) fn new_sub_context<U>(&mut self) -> MessageContext<U> {
        MessageContext {
            id_path: self.id_path.clone(),
            messages: Vec::new(),
        }
    }

    pub(crate) fn merge_sub_context<U, F: Fn(U) -> T>(
        &mut self,
        sub_context: MessageContext<U>,
        f: &F,
    ) {
        assert!(sub_context.id_path.starts_with(&self.id_path));
        let new_messages = sub_context.messages.into_iter().map(f);
        self.messages.extend(new_messages);
    }

    pub(crate) fn begin_id(&mut self, id: Id) {
        self.id_path.push(id);
    }

    pub(crate) fn end_id(&mut self) {
        let old_id = self.id_path.pop();
        debug_assert!(old_id.is_some());
    }

    pub fn id_path(&self) -> &IdPath {
        &self.id_path
    }

    pub fn push_message(&mut self, message: T) {
        self.messages.push(message);
    }

    pub fn into_messages(self) -> Vec<T> {
        self.messages
    }
}
