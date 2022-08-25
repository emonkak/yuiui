use std::sync::Arc;

use crate::state::State;

pub enum Message<S: State> {
    Pure(S::Message),
    Mutation(Box<dyn FnOnce(&mut S) -> bool + Send>),
}

impl<S: State> Message<S> {
    pub(crate) fn lift<F, PS>(self, f: Arc<F>) -> Message<PS>
    where
        S: 'static,
        F: Fn(&PS) -> &S + Sync + Send + 'static,
        PS: State,
    {
        match self {
            Message::Pure(message) => Message::Mutation(Box::new(move |state| {
                let sub_state: &mut S = unsafe { &mut *(f(state) as *const _ as *mut _) };
                sub_state.reduce(message)
            })),
            Message::Mutation(mutation) => Message::Mutation(Box::new(move |state| {
                let sub_state: &mut S = unsafe { &mut *(f(state) as *const _ as *mut _) };
                mutation(sub_state)
            })),
        }
    }
}
