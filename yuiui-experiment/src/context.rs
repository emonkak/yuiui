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

    pub fn begin(&mut self, id: Id) {
        self.id_path.push(id);
    }

    pub fn end(&mut self) -> Id {
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
    pub effects: Vec<(Vec<Id>, Effect<S>)>,
}

impl<S: State> BuildContext<S> {
    pub fn new() -> Self {
        Self {
            id_path: Vec::new(),
            effects: Vec::new(),
        }
    }

    pub fn sub_context<SS: State>(&self) -> BuildContext<SS> {
        BuildContext {
            id_path: self.id_path.clone(),
            effects: Vec::new(),
        }
    }

    pub fn begin(&mut self, id: Id) {
        self.id_path.push(id);
    }

    pub fn end(&mut self) -> Id {
        self.id_path.pop().unwrap()
    }

    pub fn push_effect(&mut self, effect: impl Into<Effect<S>>) {
        self.effects.push((self.id_path.clone(), effect.into()));
    }
}
