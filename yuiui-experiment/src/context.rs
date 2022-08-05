use yuiui_support::slot_vec::SlotVec;

pub type Id = usize;

#[derive(Debug)]
pub struct Context {
    arena: SlotVec<Vec<Id>>,
    path: Vec<Id>,
}

impl Context {
    pub fn new(depth: usize) -> Self {
        assert!(depth > 0);
        Self {
            arena: SlotVec::new(),
            path: Vec::with_capacity(depth),
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
