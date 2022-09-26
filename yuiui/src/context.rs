use crate::id::{Id, IdCounter, IdPath, IdPathBuf};

pub trait IdContext {
    fn push_id(&mut self, id: Id);

    fn pop_id(&mut self);
}

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

    pub(crate) fn next_id(&mut self) -> Id {
        self.id_counter.next()
    }

    pub fn id_path(&self) -> &IdPath {
        &self.id_path
    }
}

impl IdContext for RenderContext {
    fn push_id(&mut self, id: Id) {
        if !id.is_root() {
            self.id_path.push(id);
        }
    }

    fn pop_id(&mut self) {
        self.id_path.pop();
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

impl<T> IdContext for MessageContext<T> {
    fn push_id(&mut self, id: Id) {
        if !id.is_root() {
            self.id_path.push(id);
        }
    }

    fn pop_id(&mut self) {
        self.id_path.pop();
    }
}
