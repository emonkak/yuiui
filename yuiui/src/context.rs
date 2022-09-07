use std::sync::Arc;

use crate::effect::Effect;
use crate::event::EventResult;
use crate::id::{ComponentIndex, Id, IdPath, IdPathBuf};
use crate::state::State;

pub trait IdContext {
    fn id_path(&self) -> &IdPath;

    fn begin_view(&mut self, id: Id);

    fn end_view(&mut self) -> Id;
}

#[derive(Debug)]
pub struct RenderContext {
    id_path: IdPathBuf,
    id_counter: u64,
}

impl RenderContext {
    pub fn new() -> Self {
        Self {
            id_path: IdPathBuf::new(),
            id_counter: 0,
        }
    }

    pub fn next_identity(&mut self) -> Id {
        let id = self.id_counter;
        self.id_counter += 1;
        Id(id)
    }
}

impl IdContext for RenderContext {
    fn id_path(&self) -> &IdPath {
        &self.id_path
    }

    fn begin_view(&mut self, id: Id) {
        self.id_path.push(id);
    }

    fn end_view(&mut self) -> Id {
        self.id_path.pop().unwrap()
    }
}

pub struct CommitContext<S: State> {
    id_path: IdPathBuf,
    effects: Vec<(IdPathBuf, ComponentIndex, Effect<S>)>,
}

impl<S: State> CommitContext<S> {
    pub fn new() -> Self {
        Self {
            id_path: IdPathBuf::new(),
            effects: Vec::new(),
        }
    }

    pub fn new_sub_context<SS: State>(&self) -> CommitContext<SS> {
        CommitContext {
            id_path: self.id_path.clone(),
            effects: Vec::new(),
        }
    }

    pub fn merge_sub_context<F, SS>(&mut self, sub_context: CommitContext<SS>, f: &Arc<F>)
    where
        F: Fn(&S) -> &SS + Sync + Send + 'static,
        SS: State,
    {
        assert!(sub_context.id_path.starts_with(&self.id_path));
        let sub_effects = sub_context
            .effects
            .into_iter()
            .map(|(id_path, component_index, effect)| (id_path, component_index, effect.lift(f)));
        self.effects.extend(sub_effects);
    }

    pub fn process_result(&mut self, result: EventResult<S>, component_index: ComponentIndex) {
        for effect in result.into_effects() {
            self.effects
                .push((self.id_path.clone(), component_index, effect));
        }
    }

    pub fn into_effects(self) -> Vec<(IdPathBuf, ComponentIndex, Effect<S>)> {
        self.effects
    }
}

impl<S: State> IdContext for CommitContext<S> {
    fn id_path(&self) -> &IdPath {
        &self.id_path
    }

    fn begin_view(&mut self, id: Id) {
        self.id_path.push(id);
    }

    fn end_view(&mut self) -> Id {
        self.id_path.pop().unwrap()
    }
}
