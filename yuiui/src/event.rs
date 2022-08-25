use std::any::{Any, TypeId};
use std::collections::HashSet;

use crate::effect::Effect;
use crate::command::Command;
use crate::id::{ComponentIndex, Id, IdPath};
use crate::state::State;

pub trait Event<'event> {
    fn allowed_types() -> Vec<TypeId>;

    fn from_any(value: &'event dyn Any) -> Option<Self>
    where
        Self: Sized;

    fn from_static<T: 'static>(value: &'event T) -> Option<Self>
    where
        Self: Sized;
}

impl<'event> Event<'event> for () {
    fn allowed_types() -> Vec<TypeId> {
        Vec::new()
    }

    fn from_any(_value: &'event dyn Any) -> Option<Self>
    where
        Self: Sized,
    {
        None
    }

    fn from_static<T: 'static>(_value: &'event T) -> Option<Self>
    where
        Self: Sized,
    {
        None
    }
}

pub struct InternalEvent {
    pub id_path: IdPath,
    pub payload: Box<dyn Any>,
}

#[derive(Debug)]
pub struct EventMask {
    mask: HashSet<TypeId>,
}

impl EventMask {
    pub fn new() -> Self {
        Self {
            mask: HashSet::new(),
        }
    }

    pub fn contains(&self, type_id: &TypeId) -> bool {
        self.mask.contains(type_id)
    }

    pub fn add(&mut self, type_id: TypeId) {
        self.mask.insert(type_id);
    }

    pub fn merge(mut self, other: Self) -> Self {
        self.mask.extend(other.mask);
        self
    }
}

impl FromIterator<TypeId> for EventMask {
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = TypeId>,
    {
        EventMask {
            mask: HashSet::from_iter(iter),
        }
    }
}

impl Extend<TypeId> for EventMask {
    fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = TypeId>,
    {
        self.mask.extend(iter)
    }
}

#[must_use]
pub enum EventResult<S: State> {
    Nop,
    Effect(Effect<S>),
    Command(Command<S>),
}

#[derive(Debug, PartialEq, Eq)]
pub enum CaptureState {
    Ignored,
    Captured,
}

impl CaptureState {
    pub fn merge(self, other: Self) -> Self {
        match self {
            Self::Ignored => other,
            Self::Captured => Self::Captured,
        }
    }
}

pub struct EventContext<S: State> {
    id_path: IdPath,
    component_index: Option<ComponentIndex>,
    state_id_path: IdPath,
    state_component_index: Option<ComponentIndex>,
    pub effects: Vec<(IdPath, Option<ComponentIndex>, Effect<S>)>,
    pub commands: Vec<(IdPath, Option<ComponentIndex>, Command<S>)>,
    pub disposed_nodes: Vec<(IdPath, Option<ComponentIndex>)>,
}

impl<S: State> EventContext<S> {
    pub fn new() -> Self {
        Self {
            id_path: IdPath::new(),
            component_index: None,
            state_id_path: IdPath::new(),
            state_component_index: None,
            effects: Vec::new(),
            commands: Vec::new(),
            disposed_nodes: Vec::new(),
        }
    }

    pub fn new_sub_context<SS: State>(&self) -> EventContext<SS> {
        EventContext {
            id_path: self.id_path.clone(),
            component_index: self.component_index,
            state_id_path: self.id_path.clone(),
            state_component_index: self.component_index,
            effects: Vec::new(),
            commands: Vec::new(),
            disposed_nodes: Vec::new(),
        }
    }

    pub fn id_path(&self) -> &IdPath {
        &self.id_path
    }

    pub fn merge<SS, F>(&mut self, other: EventContext<SS>, f: F)
    where
        SS: State,
        F: Fn(Effect<SS>) -> Effect<S>,
    {
        assert!(other.id_path.starts_with(&self.id_path));
        let effects = other
            .effects
            .into_iter()
            .map(move |(id_path, component_index, effect)| (id_path, component_index, f(effect)));
        self.effects.extend(effects);
        self.disposed_nodes.extend(other.disposed_nodes);
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

    pub fn dispose_node(&mut self) {
        self.disposed_nodes
            .push((self.id_path.clone(), self.component_index));
    }

    pub fn process_result(&mut self, result: EventResult<S>) {
        match result {
            EventResult::Nop => {}
            EventResult::Effect(effect) => {
                self.effects.push((
                    self.state_id_path.clone(),
                    self.state_component_index,
                    effect,
                ));
            }
            EventResult::Command(command) => {
                self.commands.push((
                    self.state_id_path.clone(),
                    self.state_component_index,
                    command,
                ));
            }
        }
    }
}
