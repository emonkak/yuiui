use yuiui_support::slot_vec::SlotVec;

pub type Id = usize;

pub type IdPath = Vec<Id>;

#[derive(Debug)]
pub struct Context {
    arena: SlotVec<IdPath>,
    id_path: IdPath,
}

impl Context {
    pub fn new() -> Self {
        Self {
            arena: SlotVec::new(),
            id_path: Vec::new(),
        }
    }

    pub fn push(&mut self, id: Id) {
        self.id_path.push(id);
    }

    pub fn pop(&mut self) -> Id {
        self.id_path.pop().unwrap()
    }

    pub fn next_identity(&mut self) -> Id {
        self.arena.insert(self.id_path.clone())
    }

    pub fn invalidate(&mut self, id: Id) {
        self.arena.remove(id);
    }
}
