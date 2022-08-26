pub type NodeId = (Id, ComponentIndex);

pub type ComponentIndex = usize;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Id(usize);

impl Id {
    pub const ROOT: Self = Self(0);
}

#[derive(Debug, Clone)]
pub struct IdPath {
    path: Vec<Id>,
}

impl IdPath {
    pub const fn new() -> Self {
        Self { path: Vec::new() }
    }

    pub fn bottom_id(&self) -> Id {
        self.path.last().copied().unwrap_or(Id::ROOT)
    }

    pub fn top_id(&self) -> Id {
        self.path.first().copied().unwrap_or(Id::ROOT)
    }

    pub fn starts_with(&self, needle: &Self) -> bool {
        self.path.starts_with(&needle.path)
    }

    pub fn push(&mut self, id: Id) {
        self.path.push(id);
    }

    pub fn pop(&mut self) -> Id {
        self.path.pop().unwrap()
    }
}

#[derive(Debug, Clone)]
pub struct NodePath {
    id_path: IdPath,
    component_index: ComponentIndex,
}

impl NodePath {
    pub const ROOT: Self = NodePath::new(IdPath::new(), 0);

    pub const fn new(id_path: IdPath, component_index: ComponentIndex) -> Self {
        Self {
            id_path,
            component_index,
        }
    }

    pub fn id_path(&self) -> &IdPath {
        &self.id_path
    }

    pub fn component_index(&self) -> ComponentIndex {
        self.component_index
    }

    pub fn as_node_id(&self) -> NodeId {
        (self.id_path.bottom_id(), self.component_index)
    }
}

#[derive(Debug)]
pub struct IdContext {
    id_path: IdPath,
    id_counter: usize,
}

impl IdContext {
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
