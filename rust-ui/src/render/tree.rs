use std::any::TypeId;
use std::fmt;
use std::mem;

use crate::reconciler::{ReconcileResult, Reconciler};
use crate::slot_vec::SlotVec;
use crate::tree::{NodeId, Tree};
use crate::widget::element::{Children, Element, Key};
use crate::widget::null::Null;
use crate::widget::tree::{Patch, WidgetPod, WidgetTree};
use crate::widget::PolymophicWidget;

use super::RenderCycle;

#[derive(Debug)]
pub struct RenderTree<Handle> {
    tree: WidgetTree<Handle>,
    root_id: NodeId,
    render_states: SlotVec<RenderState<Handle>>,
}

#[derive(Debug)]
struct RenderState<Handle> {
    status: RenderStatus<Handle>,
}

#[derive(Debug)]
enum RenderStatus<Handle> {
    Fresh,
    Pending(Element<Handle>),
    Rendered,
    Skipped,
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
        } = &mut *self.tree[target_id];
        match mem::replace(
            &mut self.render_states[target_id].status,
            RenderStatus::Rendered,
        ) {
            RenderStatus::Fresh => {
                widget.on_render_cycle(
                    RenderCycle::WillMount(&children),
                    &mut **state.lock().unwrap(),
                    target_id,
                );
            }
            RenderStatus::Pending(element) => {
                widget.on_render_cycle(
                    RenderCycle::WillUpdate(&children, &*element.widget, &element.children),
                    &mut **state.lock().unwrap(),
                    target_id,
                );
                *widget = element.widget;
                *children = element.children;
            }
            RenderStatus::Rendered => {
                widget.on_render_cycle(
                    RenderCycle::WillUpdate(&children, &**widget, &children),
                    &mut **state.lock().unwrap(),
                    target_id,
                );
            }
            RenderStatus::Skipped => unreachable!("Skipped widget"),
        }
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
                self.render_states
                    .insert_at(node_id, RenderState::default());
                patches.push(Patch::Append(target_id, widget_pod));
            }
            ReconcileResult::Insertion(ref_id, new_element) => {
                let widget_pod = WidgetPod::from(new_element);
                let node_id = self.tree.insert_before(ref_id, widget_pod.clone());
                self.render_states
                    .insert_at(node_id, RenderState::default());
                patches.push(Patch::Insert(ref_id, widget_pod));
            }
            ReconcileResult::Update(target_id, new_element) => {
                let widget_pod = &mut self.tree[target_id];
                if widget_pod.should_update(&new_element) {
                    self.render_states[target_id].status =
                        RenderStatus::Pending(new_element.clone());
                    patches.push(Patch::Update(target_id, new_element));
                } else {
                    self.render_states[target_id].status = RenderStatus::Skipped;
                }
            }
            ReconcileResult::UpdateAndPlacement(target_id, ref_id, new_element) => {
                let widget_pod = &mut self.tree[target_id];
                if widget_pod.should_update(&new_element) {
                    self.render_states[target_id].status =
                        RenderStatus::Pending(new_element.clone());
                    patches.push(Patch::Update(target_id, new_element));
                } else {
                    self.render_states[target_id].status = RenderStatus::Skipped;
                }
                self.tree.move_position(target_id).insert_before(ref_id);
                patches.push(Patch::Placement(target_id, ref_id));
            }
            ReconcileResult::Deletion(target_id) => {
                let (node, subtree) = self.tree.detach(target_id);

                let WidgetPod {
                    widget,
                    children,
                    state,
                    ..
                } = node.into_inner();
                widget.on_render_cycle(
                    RenderCycle::WillUnmount(&children),
                    &mut **state.lock().unwrap(),
                    target_id,
                );
                self.render_states.remove(target_id);

                for (child_id, child) in subtree {
                    let WidgetPod {
                        widget,
                        children,
                        state,
                        ..
                    } = child.into_inner();
                    widget.on_render_cycle(
                        RenderCycle::WillUnmount(&children),
                        &mut **state.lock().unwrap(),
                        target_id,
                    );
                    self.render_states.remove(child_id);
                }

                patches.push(Patch::Remove(target_id));
            }
        }
    }

    fn next_render_target(&self, target_id: NodeId, initial_id: NodeId) -> Option<NodeId> {
        let mut current_node = &self.tree[target_id];

        if let Some(child_id) = self.tree[target_id].first_child() {
            if !matches!(self.render_states[child_id].status, RenderStatus::Skipped) {
                return Some(child_id);
            }
            current_node = &self.tree[child_id];
        }

        loop {
            while let Some(sibling_id) = current_node.next_sibling() {
                if !matches!(self.render_states[sibling_id].status, RenderStatus::Skipped) {
                    return Some(sibling_id);
                }
                current_node = &self.tree[sibling_id];
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
                match &render_state.status {
                    RenderStatus::Fresh => write!(f, " fresh")?,
                    RenderStatus::Pending(_) => write!(f, " pending")?,
                    RenderStatus::Rendered => write!(f, " rendered")?,
                    RenderStatus::Skipped => write!(f, " skip")?,
                }
                write!(f, ">")?;
                Ok(())
            },
            |f, _, node| write!(f, "</{}>", node.widget.name()),
        )
    }
}

impl<Handle> Default for RenderState<Handle> {
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
