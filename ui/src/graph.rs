use std::mem;

pub type NodeId = usize;

#[derive(Default)]
pub struct Graph {
    pub root: NodeId,
    pub children: Vec<Vec<NodeId>>,
    pub parent: Vec<NodeId>,
    free_list: Vec<NodeId>,
}

impl Graph {
    /// Allocate a node; it might be a previously freed id.
    pub fn alloc_node(&mut self) -> NodeId {
        if let Some(id) = self.free_list.pop() {
            return id;
        }
        let id = self.children.len();
        self.children.push(vec![]);
        self.parent.push(id);
        id
    }

    pub fn append_child(&mut self, parent: NodeId, child: NodeId) {
        self.children[parent].push(child);
        self.parent[child] = parent;
    }

    pub fn add_before(&mut self, parent: NodeId, sibling: NodeId, child: NodeId) {
        let pos = self.children[parent].iter().position(|&x| x == sibling)
            .expect("tried add_before nonexistent sibling");
        self.children[parent].insert(pos, child);
        self.parent[child] = parent;
    }

    /// Remove the child from the parent.
    ///
    /// Can panic if the graph structure is invalid. This function leaves the
    /// child in an unparented state, i.e. it can be added again.
    pub fn remove_child(&mut self, parent: NodeId, child: NodeId) {
        let ix = self.children[parent].iter().position(|&x| x == child)
            .expect("tried to remove nonexistent child");
        self.children[parent].remove(ix);
        self.parent[child] = child;
    }

    pub fn free_subtree(&mut self, node: NodeId) {
        let mut ix = self.free_list.len();
        // This is a little tricky; we're using the free list as a queue
        // for breadth-first traversal.
        self.free_list.push(node);
        while ix < self.free_list.len() {
            let node = self.free_list[ix];
            ix += 1;
            self.parent[node] = node;
            self.free_list.extend(mem::replace(&mut self.children[node], Vec::new()));
        }
    }
}

