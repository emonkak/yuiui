pub type NodeId = (Id, ComponentIndex);

pub type ComponentIndex = usize;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Id(pub(crate) usize);

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
            Self::ComponentPath(id_path, component_index) => {
                (id_path.bottom_id(), *component_index)
            }
        }
    }
}
