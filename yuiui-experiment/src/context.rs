use crate::state::{Effect, State};

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Id(usize);

pub const ROOT: Id = Id(0);

pub type ComponentIndex = usize;

#[derive(Debug)]
pub struct RenderContext {
    id_path: IdPath,
    id_counter: usize,
}

impl RenderContext {
    pub fn new() -> Self {
        Self {
            id_path: IdPath::new(),
            id_counter: 0,
        }
    }

    pub fn begin_widget(&mut self, id: Id) {
        self.id_path.push(id);
    }

    pub fn end_widget(&mut self) -> Id {
        self.id_path.pop()
    }

    pub fn next_identity(&mut self) -> Id {
        let id = self.id_counter;
        self.id_counter += 1;
        Id(id)
    }
}

pub struct EffectContext<S: State> {
    id_path: IdPath,
    component_index: Option<ComponentIndex>,
    state_id_path: IdPath,
    state_component_index: Option<ComponentIndex>,
    pub(crate) effects: Vec<(IdPath, Option<ComponentIndex>, Effect<S>)>,
}

impl<S: State> EffectContext<S> {
    pub(crate) fn new() -> Self {
        Self {
            id_path: IdPath::new(),
            component_index: None,
            state_id_path: IdPath::new(),
            state_component_index: None,
            effects: Vec::new(),
        }
    }

    pub(crate) fn new_sub_context<SS: State>(&self) -> EffectContext<SS> {
        EffectContext {
            id_path: self.id_path.clone(),
            component_index: self.component_index,
            state_id_path: self.id_path.clone(),
            state_component_index: self.component_index,
            effects: Vec::new(),
        }
    }

    pub(crate) fn begin_widget(&mut self, id: Id) {
        self.id_path.push(id);
    }

    pub(crate) fn end_widget(&mut self) -> Id {
        self.id_path.pop()
    }

    pub(crate) fn begin_components(&mut self) {
        self.component_index = Some(0);
    }

    pub(crate) fn end_components(&mut self) {
        self.component_index = None;
    }

    pub(crate) fn next_component(&mut self) {
        *self.component_index.as_mut().unwrap() += 1;
    }

    pub fn id(&self) -> Id {
        self.id_path.id()
    }

    pub fn push(&mut self, effect: impl Into<Effect<S>>) {
        self.effects.push((
            self.state_id_path.clone(),
            self.state_component_index,
            effect.into(),
        ));
    }
}

#[derive(Debug, Clone)]
pub struct IdPath(Vec<Id>);

impl IdPath {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn id(&self) -> Id {
        self.0.last().copied().unwrap_or(ROOT)
    }

    pub fn head_id(&self) -> Option<Id> {
        self.0.first().copied()
    }

    fn push(&mut self, id: Id) {
        self.0.push(id);
    }

    fn pop(&mut self) -> Id {
        self.0.pop().unwrap()
    }
}
