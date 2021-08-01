use std::any::TypeId;
use std::fmt;

use crate::lifecycle::Lifecycle;
use crate::reconciler::{ReconcileResult, Reconciler};
use crate::slot_vec::SlotVec;
use crate::tree::{NodeId, Tree};
use crate::widget::element::{Children, Element, Key};
use crate::widget::null::Null;
use crate::widget::tree::{Patch, WidgetPod, WidgetTree};
use crate::widget::PolymophicWidget;

#[derive(Debug)]
pub struct RenderTree<Handle> {
    tree: WidgetTree<Handle>,
    root_id: NodeId,
    render_states: SlotVec<RenderState>,
}

#[derive(Debug)]
struct RenderState {
    status: RenderStatus,
}

#[derive(Debug, PartialEq, Eq)]
enum RenderStatus {
    Fresh,
    Dirty,
    Skip,
}

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
enum TypedKey {
    Keyed(TypeId, Key),
    Indexed(TypeId, usize),
}

impl<Handle> RenderTree<Handle> {
    pub fn new() -> Self {
        let mut tree = Tree::new();
        let root_id = tree.attach(WidgetPod::new(Null, Vec::new()));

        let mut render_states = SlotVec::new();
        render_states.insert_at(root_id, RenderState::default());

        Self {
            tree,
            root_id,
            render_states,
        }
    }

    pub fn render(&mut self, element: Element<Handle>) -> Vec<Patch<Handle>> {
        *self.tree[self.root_id] = WidgetPod::new(Null, vec![element]);

        let mut patches = Vec::new();
        let mut current_id = self.root_id;

        while let Some(next_id) = self.render_step(current_id, self.root_id, &mut patches) {
            current_id = next_id;
        }

        patches
    }

    pub fn update(&mut self, target_id: NodeId) -> Vec<Patch<Handle>> {
        let mut patches = Vec::new();
        let mut current_id = target_id;

        while let Some(next_id) = self.render_step(current_id, target_id, &mut patches) {
            current_id = next_id;
        }

        patches
    }

    fn render_step(
        &mut self,
        target_id: NodeId,
        initial_id: NodeId,
        patches: &mut Vec<Patch<Handle>>,
    ) -> Option<NodeId> {
        let WidgetPod {
            widget,
            children,
            state,
            ..
        } = &*self.tree[target_id];
        let rendered_children =
            widget.render(children.clone(), &mut **state.lock().unwrap(), target_id);
        for result in self.reconcile_children(target_id, rendered_children) {
            self.handle_reconcile_result(target_id, result, patches);
        }
        self.next_render_target(target_id, initial_id)
    }

    fn reconcile_children(
        &mut self,
        target_id: NodeId,
        children: Children<Handle>,
    ) -> Reconciler<TypedKey, NodeId, Element<Handle>> {
        let mut old_keys: Vec<TypedKey> = Vec::new();
        let mut old_node_ids: Vec<Option<NodeId>> = Vec::new();

        for (index, (child_id, child)) in self.tree.children(target_id).enumerate() {
            let key = TypedKey::new(&*child.widget, index, child.key);
            old_keys.push(key);
            old_node_ids.push(Some(child_id));
        }

        let mut new_keys: Vec<TypedKey> = Vec::with_capacity(children.len());
        let mut new_elements: Vec<Option<Element<Handle>>> = Vec::with_capacity(children.len());

        for (index, element) in children.iter().enumerate() {
            let key = TypedKey::new(&*element.widget, index, element.key);
            new_keys.push(key);
            new_elements.push(Some(element.clone()));
        }

        Reconciler::new(old_keys, old_node_ids, new_keys, new_elements)
    }

    fn handle_reconcile_result(
        &mut self,
        target_id: NodeId,
        result: ReconcileResult<NodeId, Element<Handle>>,
        patches: &mut Vec<Patch<Handle>>,
    ) {
        match result {
            ReconcileResult::New(new_element) => {
                let widget_pod = WidgetPod::from(new_element);
                let node_id = self.tree.append_child(target_id, widget_pod.clone());
                self.handle_creation(node_id, &widget_pod);
                patches.push(Patch::Append(target_id, widget_pod));
            }
            ReconcileResult::Insertion(ref_id, new_element) => {
                let widget_pod = WidgetPod::from(new_element);
                let node_id = self.tree.insert_before(ref_id, widget_pod.clone());
                self.handle_creation(node_id, &widget_pod);
                patches.push(Patch::Insert(ref_id, widget_pod));
            }
            ReconcileResult::Update(target_id, new_element) => {
                if self.handle_update(target_id, &new_element) {
                    patches.push(Patch::Update(target_id, new_element));
                }
            }
            ReconcileResult::UpdateAndPlacement(target_id, ref_id, new_element) => {
                if self.handle_update(target_id, &new_element) {
                    patches.push(Patch::Update(target_id, new_element));
                }
                self.tree.move_position(target_id).insert_before(ref_id);
                patches.push(Patch::Placement(target_id, ref_id));
            }
            ReconcileResult::Deletion(target_id) => {
                let (node, subtree) = self.tree.detach(target_id);
                let widget_pod = node.into_inner();
                widget_pod.widget.on_render_cycle(
                    Lifecycle::OnUnmount(&widget_pod.children),
                    &mut **widget_pod.state.lock().unwrap(),
                    target_id,
                );
                self.render_states.remove(target_id);
                for (child_id, _) in subtree {
                    self.render_states.remove(child_id);
                }
                patches.push(Patch::Remove(target_id));
            }
        }
    }

    fn handle_creation(&mut self, node_id: NodeId, widget_pod: &WidgetPod<Handle>) {
        self.render_states
            .insert_at(node_id, RenderState::default());
        widget_pod.widget.on_render_cycle(
            Lifecycle::OnMount(&widget_pod.children),
            &mut **widget_pod.state.lock().unwrap(),
            node_id,
        );
    }

    fn handle_update(&mut self, target_id: NodeId, new_element: &Element<Handle>) -> bool {
        let widget_pod = &mut self.tree[target_id];
        widget_pod.widget.on_render_cycle(
            Lifecycle::OnUpdate(
                &widget_pod.children,
                &*new_element.widget,
                &new_element.children,
            ),
            &mut **widget_pod.state.lock().unwrap(),
            target_id,
        );
        if widget_pod.should_update(new_element) {
            widget_pod.update(new_element.clone());
            self.render_states[target_id].status = RenderStatus::Dirty;
            true
        } else {
            self.render_states[target_id].status = RenderStatus::Skip;
            false
        }
    }

    fn next_render_target(&self, target_id: NodeId, initial_id: NodeId) -> Option<NodeId> {
        let mut current_node = &self.tree[target_id];

        if self.render_states[target_id].status != RenderStatus::Skip {
            if let Some(first_child) = self.tree[target_id].first_child() {
                return Some(first_child);
            }
        }

        loop {
            while let Some(sibling_id) = current_node.next_sibling() {
                current_node = &self.tree[sibling_id];
                if self.render_states[target_id].status != RenderStatus::Skip {
                    return Some(sibling_id);
                }
            }

            match current_node.parent() {
                Some(parent_id) if parent_id != initial_id => {
                    current_node = &self.tree[parent_id];
                }
                _ => break,
            }
        }

        None
    }
}

impl<Handle> fmt::Display for RenderTree<Handle> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.tree.format(
            f,
            self.root_id,
            |f, node_id, node| {
                let render_state = &self.render_states[node_id];
                write!(f, "<{}", node.widget.name())?;
                write!(f, " id=\"{}\"", node_id)?;
                if let Some(key) = node.key {
                    write!(f, " key=\"{}\"", key)?;
                }
                write!(f, " status=\"{:?}\"", render_state.status)?;
                write!(f, ">")?;
                Ok(())
            },
            |f, _, node| write!(f, "</{}>", node.widget.name()),
        )
    }
}

impl Default for RenderState {
    fn default() -> Self {
        Self {
            status: RenderStatus::Fresh,
        }
    }
}

impl TypedKey {
    fn new<Handle>(widget: &dyn PolymophicWidget<Handle>, index: usize, key: Option<Key>) -> Self {
        match key {
            Some(key) => Self::Keyed(widget.as_any().type_id(), key),
            None => Self::Indexed(widget.as_any().type_id(), index),
        }
    }
}
