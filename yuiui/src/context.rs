use std::any::Any;
use std::rc::Rc;

use crate::effect::StateScope;
use crate::id::{ComponentIndex, Id, IdPath, IdPathBuf};

pub trait IdContext {
    fn id_path(&self) -> &IdPath;

    fn begin_id(&mut self, id: Id);

    fn end_id(&mut self) -> Id;

    fn id_guard<F: FnOnce(&mut Self) -> T, T>(&mut self, id: Id, f: F) -> T {
        self.begin_id(id);
        let result = f(self);
        self.end_id();
        result
    }
}

#[derive(Debug)]
pub struct RenderContext {
    id_path: IdPathBuf,
    id_counter: u64,
    env_stack: Vec<(Id, Rc<dyn Any>)>,
}

impl RenderContext {
    pub fn new() -> Self {
        Self {
            id_path: IdPathBuf::new(),
            id_counter: 0,
            env_stack: Vec::new(),
        }
    }

    pub fn next_id(&mut self) -> Id {
        let id = self.id_counter;
        self.id_counter += 1;
        Id(id)
    }

    pub fn with_id<F: FnOnce(Id, &mut Self) -> T, T>(&mut self, f: F) -> T {
        let id = self.next_id();
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
        self.env_stack
            .push((Id::from_bottom(self.id_path.as_slice()), value))
    }
}

impl IdContext for RenderContext {
    fn id_path(&self) -> &IdPath {
        &self.id_path
    }

    fn begin_id(&mut self, id: Id) {
        self.id_path.push(id);
    }

    fn end_id(&mut self) -> Id {
        let previous_id = self.id_path.pop().unwrap();

        while let Some((id, _)) = self.env_stack.last() {
            if *id == previous_id {
                self.env_stack.pop();
            } else {
                break;
            }
        }

        previous_id
    }
}

#[derive(Debug, Clone)]
pub struct EffectContext {
    id_path: IdPathBuf,
    component_index: ComponentIndex,
    state_scope: StateScope,
}

impl EffectContext {
    pub fn new() -> Self {
        Self {
            id_path: IdPathBuf::new(),
            component_index: 0,
            state_scope: StateScope::Global,
        }
    }

    pub fn new_sub_context(&self) -> EffectContext {
        EffectContext {
            id_path: self.id_path.clone(),
            component_index: self.component_index,
            state_scope: StateScope::Partial(self.id_path.clone(), self.component_index),
        }
    }

    pub fn begin_effect(&mut self, component_index: ComponentIndex) {
        self.component_index = component_index;
    }

    pub fn state_scope(&self) -> &StateScope {
        &self.state_scope
    }
}

impl IdContext for EffectContext {
    fn id_path(&self) -> &IdPath {
        &self.id_path
    }

    fn begin_id(&mut self, id: Id) {
        self.id_path.push(id);
        self.component_index = 0;
    }

    fn end_id(&mut self) -> Id {
        self.id_path.pop().unwrap()
    }
}
