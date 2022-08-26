use crate::sequence::TraverseContext;

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
pub enum NodePath {
    WidgetPath(IdPath),
    ComponentPath(IdPath, ComponentIndex),
}

impl NodePath {
    pub fn new(id_path: IdPath, component_index: Option<ComponentIndex>) -> Self {
        if let Some(component_index) = component_index {
            Self::ComponentPath(id_path, component_index)
        } else {
            Self::WidgetPath(id_path)
        }
    }

    pub fn id_path(&self) -> &IdPath {
        match self {
            Self::WidgetPath(id_path) => &id_path,
            Self::ComponentPath(id_path, _) => &id_path,
        }
    }

    pub fn as_node_id(&self) -> NodeId {
        match self {
            Self::WidgetPath(id_path) => (id_path.bottom_id(), 0),
            Self::ComponentPath(id_path, component_index) => (id_path.bottom_id(), *component_index),
        }
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

    pub fn next_identity(&mut self) -> Id {
        let id = self.id_counter;
        self.id_counter += 1;
        Id(id)
    }
}

impl TraverseContext for IdContext {
    fn id_path(&self) -> &IdPath {
        &self.id_path
    }

    fn begin_widget(&mut self, id: Id) {
        self.id_path.push(id);
    }

    fn end_widget(&mut self) -> Id {
        self.id_path.pop()
    }
}
