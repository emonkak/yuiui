use futures::stream::BoxStream;

use crate::event::EventResult;
use crate::id::{ComponentIndex, Id, IdPath};
use crate::state::State;

pub enum Effect<S: State> {
    Message(S::Message),
    Mutation(Box<dyn Mutation<S> + Send>),
    Command(BoxStream<'static, Effect<S>>),
}

pub trait Mutation<S> {
    fn apply(&mut self, state: &mut S) -> bool;
}

impl<S: State> Mutation<S> for Box<dyn Mutation<S> + Send> {
    fn apply(&mut self, state: &mut S) -> bool {
        self.as_mut().apply(state)
    }
}

impl<S: State> Mutation<S> for Option<S::Message> {
    fn apply(&mut self, state: &mut S) -> bool {
        state.reduce(self.take().unwrap())
    }
}

pub struct EffectContext<S: State> {
    id_path: IdPath,
    component_index: Option<ComponentIndex>,
    state_id_path: IdPath,
    state_component_index: Option<ComponentIndex>,
    unit_of_work: UnitOfWork<S>,
}

impl<S: State> EffectContext<S> {
    pub fn new() -> Self {
        Self {
            id_path: IdPath::new(),
            component_index: None,
            state_id_path: IdPath::new(),
            state_component_index: None,
            unit_of_work: UnitOfWork::new(),
        }
    }

    pub fn new_sub_context<SS: State>(&self) -> EffectContext<SS> {
        EffectContext {
            id_path: self.id_path.clone(),
            component_index: self.component_index,
            state_id_path: self.id_path.clone(),
            state_component_index: self.component_index,
            unit_of_work: UnitOfWork::new(),
        }
    }

    pub fn id_path(&self) -> &IdPath {
        &self.id_path
    }

    pub fn merge_unit_of_work<SS, F>(&mut self, unit_of_work: UnitOfWork<SS>, f: F)
    where
        SS: State,
        F: Fn(Effect<SS>) -> Effect<S>,
    {
        self.unit_of_work.merge(unit_of_work, f);
    }

    pub fn into_unit_of_work(self) -> UnitOfWork<S> {
        self.unit_of_work
    }

    pub fn begin_widget(&mut self, id: Id) {
        self.id_path.push(id);
    }

    pub fn end_widget(&mut self) -> Id {
        self.id_path.pop()
    }

    pub fn begin_components(&mut self) {
        self.component_index = Some(0);
    }

    pub fn next_component(&mut self) {
        *self.component_index.as_mut().unwrap() += 1;
    }

    pub fn end_components(&mut self) {
        self.component_index = None;
    }

    pub fn mark_unmounted(&mut self) {
        self.unit_of_work
            .unmounted_nodes
            .push((self.id_path.clone(), self.component_index));
    }

    pub fn process(&mut self, result: EventResult<S>) {
        match result {
            EventResult::Nop => {}
            EventResult::Effect(effect) => {
                self.unit_of_work.effects.push((
                    self.state_id_path.clone(),
                    self.state_component_index,
                    effect,
                ));
            }
        }
    }
}

pub struct UnitOfWork<S: State> {
    pub effects: Vec<(IdPath, Option<ComponentIndex>, Effect<S>)>,
    pub unmounted_nodes: Vec<(IdPath, Option<ComponentIndex>)>,
}

impl<S: State> UnitOfWork<S> {
    pub fn new() -> Self {
        Self {
            effects: Vec::new(),
            unmounted_nodes: Vec::new(),
        }
    }

    pub fn merge<SS, F>(&mut self, other: UnitOfWork<SS>, f: F)
    where
        SS: State,
        F: Fn(Effect<SS>) -> Effect<S>,
    {
        let effects = other
            .effects
            .into_iter()
            .map(move |(id_path, component_index, effect)| (id_path, component_index, f(effect)));
        self.effects.extend(effects);
        self.unmounted_nodes.extend(other.unmounted_nodes);
    }
}
