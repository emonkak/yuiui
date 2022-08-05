use yuiui_support::slot_vec::SlotVec;

pub type Id = usize;

#[derive(Debug)]
pub struct Context {
    path: Vec<Id>,
    arena: SlotVec<Vec<Id>>,
}

impl Context {
    pub fn new() -> Self {
        Self {
            path: Vec::new(),
            arena: SlotVec::new(),
        }
    }

    pub fn push(&mut self, id: Id) {
        self.path.push(id);
    }

    pub fn pop(&mut self) -> Id {
        self.path.pop().unwrap()
    }

    pub fn next_identity(&mut self) -> Id {
        self.arena.insert(self.path.clone())
    }

    pub fn invalidate(&mut self, id: Id) {
        self.arena.remove(id);
    }
}
