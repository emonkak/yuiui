use std::any::Any;
use std::rc::Rc;

use crate::id::{Depth, Id, IdCounter, IdPath, IdPathBuf};

#[derive(Debug)]
pub struct RenderContext {
    id_path: IdPathBuf,
    id_counter: IdCounter,
    env_stack: Vec<(Id, Rc<dyn Any>)>,
}

impl RenderContext {
    pub fn new() -> Self {
        Self {
            id_path: IdPathBuf::new(),
            id_counter: IdCounter::new(),
            env_stack: Vec::new(),
        }
    }

    pub fn id_path(&self) -> &IdPath {
        &self.id_path
    }

    pub fn begin_id(&mut self, id: Id) {
        self.id_path.push(id);
    }

    pub fn end_id(&mut self) {
        let id = self.id_path.pop().unwrap();

        while let Some((env_id, _)) = self.env_stack.last() {
            if *env_id == id {
                self.env_stack.pop();
            } else {
                break;
            }
        }
    }

    pub fn with_id<F: FnOnce(Id, &mut Self) -> T, T>(&mut self, f: F) -> T {
        let id = self.id_counter.next();
        self.id_path.push(id);
        let result = f(id, self);
        self.id_path.pop();
        result
    }

    pub fn get_env<T: 'static>(&self) -> Option<&T> {
        for (_, env) in self.env_stack.iter().rev() {
            if let Some(value) = env.downcast_ref() {
                return Some(value);
            }
        }
        None
    }

    pub fn push_env(&mut self, value: Rc<dyn Any>) {
        self.env_stack.push((Id::from_bottom(&self.id_path), value))
    }
}

#[derive(Debug, Clone)]
pub struct EffectContext {
    id_path: IdPathBuf,
    depth: Depth,
    state_scope: StateScope,
}

impl EffectContext {
    pub fn new() -> Self {
        Self {
            id_path: IdPathBuf::new(),
            depth: 0,
            state_scope: StateScope::Whole,
        }
    }

    pub fn new_sub_context(&self) -> EffectContext {
        EffectContext {
            id_path: self.id_path.clone(),
            depth: self.depth,
            state_scope: StateScope::Subtree(self.id_path.clone(), self.depth),
        }
    }

    pub fn begin_id(&mut self, id: Id) {
        self.id_path.push(id);
        self.depth = 0;
    }

    pub fn end_id(&mut self) {
        self.id_path.pop();
    }

    pub fn set_depth(&mut self, depth: Depth) {
        self.depth = depth;
    }

    pub fn id_path(&self) -> &IdPath {
        &self.id_path
    }

    pub fn depth(&self) -> Depth {
        self.depth
    }

    pub fn state_scope(&self) -> &StateScope {
        &self.state_scope
    }
}

#[derive(Debug, Clone)]
pub enum StateScope {
    Whole,
    Subtree(IdPathBuf, Depth),
}

impl StateScope {
    pub fn normalize(self) -> (IdPathBuf, Depth) {
        match self {
            StateScope::Whole => (IdPathBuf::new(), 0),
            StateScope::Subtree(id_path, depth) => (id_path, depth),
        }
    }
}
