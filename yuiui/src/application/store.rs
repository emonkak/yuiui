#[derive(Debug)]
pub struct Store<State, Reducer> {
    state: State,
    reducer: Reducer,
}

impl<State, Reducer> Store<State, Reducer> {
    pub fn new<Message>(initial_state: State, reducer: Reducer) -> Self
    where
        Reducer: Fn(&mut State, Message) -> bool,
    {
        Self {
            state: initial_state,
            reducer,
        }
    }

    pub fn state(&self) -> &State {
        &self.state
    }

    pub fn dispatch<Message>(&mut self, message: Message) -> bool
    where
        Reducer: Fn(&mut State, Message) -> bool,
    {
        (self.reducer)(&mut self.state, message)
    }
}
