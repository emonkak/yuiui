use std::sync::Arc;

use crate::effect::{Effect, EffectPath};
use crate::event::EventResult;
use crate::id::{ComponentIndex, Id, IdPath};
use crate::state::State;

pub trait IdContext {
    fn id_path(&self) -> &IdPath;

    fn begin_widget(&mut self, id: Id);

    fn end_widget(&mut self) -> Id;
}

#[derive(Debug)]
pub struct RenderContext {
    id_path: IdPath,
    id_counter: u64,
}

impl RenderContext {
    pub fn new() -> Self {
        Self {
            id_path: IdPath::new(),
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

    fn begin_widget(&mut self, id: Id) {
        self.id_path.push(id);
    }

    fn end_widget(&mut self) -> Id {
        self.id_path.pop().unwrap()
    }
}

pub struct EffectContext<S: State> {
    effect_path: EffectPath,
    effects: Vec<(EffectPath, Effect<S>)>,
}

impl<S: State> EffectContext<S> {
    pub fn new() -> Self {
        Self {
            effect_path: EffectPath::new(),
            effects: Vec::new(),
        }
    }

    pub fn effect_path(&self) -> &EffectPath {
        &self.effect_path
    }

    pub fn new_sub_context<SS: State>(&self) -> EffectContext<SS> {
        EffectContext {
            effect_path: self.effect_path.new_sub_path(),
            effects: Vec::new(),
        }
    }

    pub fn set_component_index(&mut self, component_index: ComponentIndex) {
        self.effect_path.component_index = component_index;
    }

    pub fn increment_component_index(&mut self) {
        self.effect_path.component_index += 1;
    }

    pub fn merge_sub_context<F, SS>(&mut self, sub_context: EffectContext<SS>, f: &Arc<F>)
    where
        F: Fn(&S) -> &SS + Sync + Send + 'static,
        SS: State,
    {
        assert!(sub_context
            .effect_path
            .id_path
            .starts_with(&self.effect_path.id_path));
        let sub_effects = sub_context
            .effects
            .into_iter()
            .map(|(effect_path, effect)| (effect_path, effect.lift(f)));
        self.effects.extend(sub_effects);
    }

    pub fn process_result(&mut self, result: EventResult<S>) {
        for effect in result.into_effects() {
            let effect_path = self.effect_path.clone();
            self.effects.push((effect_path, effect));
        }
    }

    pub fn into_effects(self) -> Vec<(EffectPath, Effect<S>)> {
        self.effects
    }
}

impl<S: State> IdContext for EffectContext<S> {
    fn id_path(&self) -> &IdPath {
        &self.effect_path.id_path
    }

    fn begin_widget(&mut self, id: Id) {
        self.effect_path.id_path.push(id);
        self.effect_path.component_index = 0;
    }

    fn end_widget(&mut self) -> Id {
        self.effect_path.id_path.pop().unwrap()
    }
}
