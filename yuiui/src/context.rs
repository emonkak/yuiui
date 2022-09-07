use std::any::Any;
use std::rc::Rc;
use std::sync::Arc;

use crate::effect::Effect;
use crate::event::EventResult;
use crate::id::{ComponentIndex, Id, IdPath, IdPathBuf, StateScope};
use crate::state::State;

pub trait IdContext {
    fn id_path(&self) -> &IdPath;

    fn with_view<F: FnOnce(&mut Self) -> T, T>(
        &mut self,
        id: Id,
        env: &Option<Rc<dyn Any>>,
        f: F,
    ) -> T {
        self.begin_view(id, env);
        let result = f(self);
        self.end_view();
        result
    }

    fn begin_view(&mut self, id: Id, env: &Option<Rc<dyn Any>>);

    fn end_view(&mut self) -> Id;
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

    pub fn next_identity(&mut self) -> Id {
        let id = self.id_counter;
        self.id_counter += 1;
        Id(id)
    }

    pub fn with_identity<F: FnOnce(Id, &mut Self) -> T, T>(&mut self, f: F) -> T {
        let id = self.next_identity();
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
}

impl IdContext for RenderContext {
    fn id_path(&self) -> &IdPath {
        &self.id_path
    }

    fn begin_view(&mut self, id: Id, env: &Option<Rc<dyn Any>>) {
        self.id_path.push(id);
        if let Some(value) = env {
            self.env_stack
                .push((Id::from_bottom(self.id_path.as_slice()), value.clone()))
        }
    }

    fn end_view(&mut self) -> Id {
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

pub struct CommitContext<S: State> {
    id_path: IdPathBuf,
    component_index: ComponentIndex,
    scope: StateScope,
    effects: Vec<(IdPathBuf, ComponentIndex, StateScope, Effect<S>)>,
}

impl<S: State> CommitContext<S> {
    pub fn new() -> Self {
        Self {
            id_path: IdPathBuf::new(),
            component_index: 0,
            scope: StateScope::Global,
            effects: Vec::new(),
        }
    }

    pub fn new_sub_context<SS: State>(&self) -> CommitContext<SS> {
        CommitContext {
            id_path: self.id_path.clone(),
            component_index: self.component_index,
            scope: StateScope::Partial(self.id_path.clone(), self.component_index),
            effects: Vec::new(),
        }
    }

    pub fn merge_sub_context<F, SS>(&mut self, sub_context: CommitContext<SS>, f: &Arc<F>)
    where
        F: Fn(&S) -> &SS + Sync + Send + 'static,
        SS: State,
    {
        assert!(sub_context.id_path.starts_with(&self.id_path));
        let sub_effects =
            sub_context
                .effects
                .into_iter()
                .map(|(id_path, component_index, scope, effect)| {
                    (id_path, component_index, scope, effect.lift(f))
                });
        self.effects.extend(sub_effects);
    }

    pub fn process_result(&mut self, result: EventResult<S>, component_index: ComponentIndex) {
        for effect in result.into_effects() {
            self.effects.push((
                self.id_path.clone(),
                component_index,
                self.scope.clone(),
                effect,
            ));
        }
        self.component_index = component_index;
    }

    pub fn into_effects(self) -> Vec<(IdPathBuf, ComponentIndex, StateScope, Effect<S>)> {
        self.effects
    }
}

impl<S: State> IdContext for CommitContext<S> {
    fn id_path(&self) -> &IdPath {
        &self.id_path
    }

    fn begin_view(&mut self, id: Id, _env: &Option<Rc<dyn Any>>) {
        self.id_path.push(id);
        self.component_index = 0;
    }

    fn end_view(&mut self) -> Id {
        self.id_path.pop().unwrap()
    }
}
