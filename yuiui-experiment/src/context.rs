use crate::state::{Effect, State};

pub type Id = usize;

#[derive(Debug)]
pub struct RenderContext {
    id_path: Vec<Id>,
    id_counter: usize,
}

impl RenderContext {
    pub fn new() -> Self {
        Self {
            id_counter: 0,
            id_path: Vec::new(),
        }
    }

    pub fn begin_widget(&mut self, id: Id) {
        self.id_path.push(id);
    }

    pub fn end_widget(&mut self) -> Id {
        self.id_path.pop().unwrap()
    }

    pub fn next_identity(&mut self) -> Id {
        let id = self.id_counter;
        self.id_counter += 1;
        id
    }
}

pub struct BuildContext<S: State> {
    id_path: Vec<Id>,
    component_index: Option<usize>,
    pub effects: Vec<(Vec<Id>, Effect<S>)>,
}

impl<S: State> BuildContext<S> {
    pub fn new() -> Self {
        Self {
            id_path: Vec::new(),
            component_index: Some(0),
            effects: Vec::new(),
        }
    }

    pub fn sub_context<SS: State>(&self) -> BuildContext<SS> {
        BuildContext {
            id_path: self.id_path.clone(),
            component_index: self.component_index,
            effects: Vec::new(),
        }
    }

    pub fn begin_widget(&mut self, id: Id) {
        self.id_path.push(id);
    }

    pub fn end_widget(&mut self) -> Id {
        self.id_path.pop().unwrap()
    }

    pub fn begin_components(&mut self) {
        self.component_index = Some(0);
    }

    pub fn end_components(&mut self) {
        self.component_index = None;
    }

    pub fn next_component(&mut self) {
        *self.component_index.as_mut().unwrap() += 1;
    }

    pub fn push_effect(&mut self, effect: impl Into<Effect<S>>) {
        self.effects.push((self.id_path.clone(), effect.into()));
    }
}
