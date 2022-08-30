use std::sync::Arc;

use crate::effect::{Effect, EffectPath};
use crate::event::EventResult;
use crate::id::{ComponentIndex, Id, IdPath};
use crate::state::State;

pub trait IdContext {
    fn id_path(&self) -> &IdPath;

    fn component_index(&self) -> Option<ComponentIndex>;

    fn begin_widget(&mut self, id: Id);

    fn end_widget(&mut self) -> Id;

    fn begin_components(&mut self);

    fn next_component(&mut self);

    fn end_components(&mut self);
}

#[derive(Debug)]
pub struct RenderContext {
    id_path: IdPath,
    component_index: Option<ComponentIndex>,
    id_counter: u64,
}

impl RenderContext {
    pub fn new() -> Self {
        Self {
            id_path: IdPath::new(),
            component_index: None,
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

    fn component_index(&self) -> Option<ComponentIndex> {
        self.component_index
    }

    fn begin_widget(&mut self, id: Id) {
        self.id_path.push(id);
    }

    fn end_widget(&mut self) -> Id {
        self.id_path.pop()
    }

    fn begin_components(&mut self) {
        self.component_index = Some(0);
    }

    fn next_component(&mut self) {
        *self.component_index.as_mut().unwrap() += 1;
    }

    fn end_components(&mut self) {
        self.component_index = None;
    }
}

pub struct EffectContext<S: State> {
    id_path: IdPath,
    component_index: Option<ComponentIndex>,
    state_id_path: IdPath,
    state_component_index: Option<ComponentIndex>,
    effects: Vec<(EffectPath, Effect<S>)>,
}

impl<S: State> EffectContext<S> {
    pub fn new() -> Self {
        Self {
            id_path: IdPath::new(),
            component_index: None,
            state_id_path: IdPath::new(),
            state_component_index: None,
            effects: Vec::new(),
        }
    }

    pub fn new_sub_context<SS: State>(&self) -> EffectContext<SS> {
        EffectContext {
            id_path: self.id_path.clone(),
            component_index: self.component_index,
            state_id_path: self.id_path.clone(),
            state_component_index: self.component_index,
            effects: Vec::new(),
        }
    }

    pub fn merge_sub_context<F, SS>(&mut self, sub_context: EffectContext<SS>, f: &Arc<F>)
    where
        F: Fn(&S) -> &SS + Sync + Send + 'static,
        SS: State,
    {
        assert!(sub_context.id_path.starts_with(&self.id_path));

        let sub_effects = sub_context
            .effects
            .into_iter()
            .map(|(effect_path, effect)| (effect_path, effect.lift(f)));
        self.effects.extend(sub_effects);
    }

    pub fn process_result(&mut self, result: EventResult<S>) {
        for effect in result.into_effects() {
            let path = EffectPath {
                source_path: (self.id_path.clone(), self.component_index),
                state_path: (self.state_id_path.clone(), self.state_component_index),
            };
            self.effects.push((path, effect));
        }
    }

    pub fn into_effects(self) -> Vec<(EffectPath, Effect<S>)> {
        self.effects
    }
}

impl<S: State> IdContext for EffectContext<S> {
    fn id_path(&self) -> &IdPath {
        &self.id_path
    }

    fn component_index(&self) -> Option<ComponentIndex> {
        self.component_index
    }

    fn begin_widget(&mut self, id: Id) {
        self.id_path.push(id);
    }

    fn end_widget(&mut self) -> Id {
        self.id_path.pop()
    }

    fn begin_components(&mut self) {
        self.component_index = Some(0);
    }

    fn next_component(&mut self) {
        *self.component_index.as_mut().unwrap() += 1;
    }

    fn end_components(&mut self) {
        self.component_index = None;
    }
}
