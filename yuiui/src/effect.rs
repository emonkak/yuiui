use std::sync::Arc;

use crate::command::Command;
use crate::id::{NodeId, NodePath};
use crate::state::State;

pub enum Effect<S: State> {
    Message(S::Message),
    Mutation(Box<dyn FnOnce(&mut S) -> bool + Send>),
    Command(Command<S>),
}

impl<S: State> Effect<S> {
    pub(crate) fn lift<F, NS>(self, f: &Arc<F>) -> Effect<NS>
    where
        F: Fn(&NS) -> &S + Sync + Send + 'static,
        NS: State,
    {
        match self {
            Self::Message(message) => {
                let f = f.clone();
                Effect::Mutation(Box::new(move |state| {
                    let sub_state: &mut S = unsafe { &mut *(f(state) as *const _ as *mut _) };
                    sub_state.reduce(message)
                }))
            }
            Self::Mutation(mutation) => {
                let f = f.clone();
                Effect::Mutation(Box::new(move |state| {
                    let sub_state: &mut S = unsafe { &mut *(f(state) as *const _ as *mut _) };
                    mutation(sub_state)
                }))
            }
            Self::Command(command) => {
                let f = f.clone();
                let command = command.map(move |effect| effect.lift(&f));
                Effect::Command(command)
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct EffectPath {
    source_path: NodePath,
    state_path: NodePath,
}

impl EffectPath {
    pub fn new(source_path: NodePath, state_path: NodePath) -> Self {
        Self {
            source_path,
            state_path,
        }
    }

    pub fn source_path(&self) -> &NodePath {
        &self.source_path
    }

    pub fn source_id(&self) -> NodeId {
        let (id_path, component_index) = &self.source_path;
        (id_path.bottom_id(), *component_index)
    }

    pub fn state_path(&self) -> &NodePath {
        &self.state_path
    }
}
