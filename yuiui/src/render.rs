use crate::component_node::ComponentStack;
use crate::state::State;
use crate::view::View;
use crate::widget_node::WidgetNode;

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
            Self::ComponentPath(id_path, component_index) => {
                (id_path.bottom_id(), *component_index)
            }
        }
    }
}

#[derive(Debug)]
pub struct RenderContext {
    id_path: IdPath,
    id_counter: usize,
}

impl RenderContext {
    pub fn new() -> Self {
        Self {
            id_path: IdPath::new(),
            id_counter: 0,
        }
    }

    pub fn id_path(&self) -> &IdPath {
        &self.id_path
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

pub trait RenderContextSeq<S: State, E> {
    fn for_each<V: RenderContextVisitor>(
        &mut self,
        visitor: &mut V,
        state: &S,
        env: &E,
        context: &mut RenderContext,
    );

    fn search<V: RenderContextVisitor>(
        &mut self,
        id_path: &IdPath,
        visitor: &mut V,
        state: &S,
        env: &E,
        context: &mut RenderContext,
    ) -> bool;
}

pub trait RenderContextVisitor {
    fn visit<V, CS, S, E>(
        &mut self,
        node: &mut WidgetNode<V, CS, S, E>,
        state: &S,
        env: &E,
        context: &mut RenderContext,
    ) where
        V: View<S, E>,
        CS: ComponentStack<S, E, View = V>,
        S: State;
}
