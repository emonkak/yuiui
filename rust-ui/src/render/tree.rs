use std::any::TypeId;
use std::fmt;
use std::mem;
use std::sync::mpsc::Sender;

use crate::support::slot_vec::SlotVec;
use crate::widget::element::{Children, Element, Key};
use crate::widget::null::Null;
use crate::widget::{
    create_widget_tree, PolymophicWidget, WidgetId, WidgetPod, WidgetTree, WidgetTreePatch,
};

use super::reconciler::{ReconcileResult, Reconciler};
use super::context::RenderContext;

#[derive(Debug)]
pub struct RenderTree<Renderer> {
    tree: WidgetTree<Renderer>,
    root_id: WidgetId,
    render_states: SlotVec<RenderState<Renderer>>,
    update_sender: Sender<WidgetId>,
}

#[derive(Debug)]
struct RenderState<Renderer> {
    status: RenderStatus<Renderer>,
}

#[derive(Debug)]
enum RenderStatus<Renderer> {
    Fresh,
    Pending(Element<Renderer>),
    Rendered,
    Skipped,
}

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
enum TypedKey {
    Keyed(TypeId, Key),
    Indexed(TypeId, usize),
}

impl<Renderer> RenderTree<Renderer> {
    pub fn new(update_sender: Sender<WidgetId>) -> Self {
        let (tree, root_id) = create_widget_tree();
        let mut render_states = SlotVec::new();

        render_states.insert_at(root_id, RenderState::default());

        Self {
            tree,
            root_id,
            render_states,
            update_sender,
        }
    }

    #[inline]
    pub fn root_id(&self) -> WidgetId {
        self.root_id
    }

    pub fn render(&mut self, element: Element<Renderer>) -> Vec<WidgetTreePatch<Renderer>> {
        *self.tree[self.root_id] = WidgetPod::new(Null, vec![element]);

        let mut patches = Vec::new();
        let mut current_id = self.root_id;

        while let Some(next_id) = self.render_step(current_id, self.root_id, &mut patches) {
            current_id = next_id;
        }

        patches
    }

    pub fn update(&mut self, target_id: WidgetId) -> Vec<WidgetTreePatch<Renderer>> {
        let mut patches = Vec::new();
        let mut current_id = target_id;

        while let Some(next_id) = self.render_step(current_id, target_id, &mut patches) {
            current_id = next_id;
        }

        patches
    }

    fn render_step(
        &mut self,
        target_id: WidgetId,
        initial_id: WidgetId,
        patches: &mut Vec<WidgetTreePatch<Renderer>>,
    ) -> Option<WidgetId> {
        let WidgetPod {
            widget,
            children,
            state,
            ..
        } = &mut *self.tree[target_id];

        let old_status = mem::replace(
            &mut self.render_states[target_id].status,
            RenderStatus::Rendered,
        );
        match old_status {
            RenderStatus::Pending(element) => {
                *widget = element.widget;
                *children = element.children;
            }
            RenderStatus::Skipped => unreachable!("Skipped widget"),
            RenderStatus::Fresh | RenderStatus::Rendered => {}
        }

        let context = RenderContext::new(target_id, self.update_sender.clone());
        let rendered_children = widget
            .clone()
            .render(children.clone(), state.clone(), context);

        for result in self.reconcile_children(target_id, rendered_children) {
            self.handle_reconcile_result(target_id, result, patches);
        }

        self.next_render_target(target_id, initial_id)
    }

    fn reconcile_children(
        &mut self,
        target_id: WidgetId,
        children: Children<Renderer>,
    ) -> Reconciler<TypedKey, WidgetId, Element<Renderer>> {
        let mut old_keys: Vec<TypedKey> = Vec::new();
        let mut old_widget_ids: Vec<Option<WidgetId>> = Vec::new();

        for (index, (child_id, child)) in self.tree.children(target_id).enumerate() {
            let key = TypedKey::new(&*child.widget, index, child.key);
            old_keys.push(key);
            old_widget_ids.push(Some(child_id));
        }

        let mut new_keys: Vec<TypedKey> = Vec::with_capacity(children.len());
        let mut new_elements: Vec<Option<Element<Renderer>>> = Vec::with_capacity(children.len());

        for (index, element) in children.iter().enumerate() {
            let key = TypedKey::new(&*element.widget, index, element.key);
            new_keys.push(key);
            new_elements.push(Some(element.clone()));
        }

        Reconciler::new(old_keys, old_widget_ids, new_keys, new_elements)
    }

    fn handle_reconcile_result(
        &mut self,
        target_id: WidgetId,
        result: ReconcileResult<WidgetId, Element<Renderer>>,
        patches: &mut Vec<WidgetTreePatch<Renderer>>,
    ) {
        match result {
            ReconcileResult::New(new_element) => {
                let widget_pod = WidgetPod::from(new_element);
                let widget_id = self.tree.append_child(target_id, widget_pod.clone());
                self.render_states
                    .insert_at(widget_id, RenderState::default());
                patches.push(WidgetTreePatch::Append(target_id, widget_pod));
            }
            ReconcileResult::Insertion(ref_id, new_element) => {
                let widget_pod = WidgetPod::from(new_element);
                let widget_id = self.tree.insert_before(ref_id, widget_pod.clone());
                self.render_states
                    .insert_at(widget_id, RenderState::default());
                patches.push(WidgetTreePatch::Insert(ref_id, widget_pod));
            }
            ReconcileResult::Update(target_id, new_element) => {
                let widget_pod = &mut self.tree[target_id];
                if widget_pod.should_update(&new_element) {
                    self.render_states[target_id].status =
                        RenderStatus::Pending(new_element.clone());
                    patches.push(WidgetTreePatch::Update(target_id, new_element));
                } else {
                    self.render_states[target_id].status = RenderStatus::Skipped;
                }
            }
            ReconcileResult::UpdateAndPlacement(target_id, ref_id, new_element) => {
                let widget_pod = &mut self.tree[target_id];
                if widget_pod.should_update(&new_element) {
                    self.render_states[target_id].status =
                        RenderStatus::Pending(new_element.clone());
                    patches.push(WidgetTreePatch::Update(target_id, new_element));
                } else {
                    self.render_states[target_id].status = RenderStatus::Skipped;
                }
                self.tree.move_position(target_id).insert_before(ref_id);
                patches.push(WidgetTreePatch::Placement(target_id, ref_id));
            }
            ReconcileResult::Deletion(target_id) => {
                let (_, subtree) = self.tree.detach(target_id);

                self.render_states.remove(target_id);

                for (child_id, _) in subtree {
                    self.render_states.remove(child_id);
                }

                patches.push(WidgetTreePatch::Remove(target_id));
            }
        }
    }

    fn next_render_target(&self, target_id: WidgetId, initial_id: WidgetId) -> Option<WidgetId> {
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

impl<Renderer> fmt::Display for RenderTree<Renderer> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.tree.format(
            f,
            self.root_id,
            |f, widget_id, node| {
                let render_state = &self.render_states[widget_id];
                write!(f, "<{}", node.widget.name())?;
                write!(f, " id=\"{}\"", widget_id)?;
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

impl<Renderer> Default for RenderState<Renderer> {
    fn default() -> Self {
        Self {
            status: RenderStatus::Fresh,
        }
    }
}

impl TypedKey {
    fn new<Renderer>(
        widget: &dyn PolymophicWidget<Renderer>,
        index: usize,
        key: Option<Key>,
    ) -> Self {
        match key {
            Some(key) => Self::Keyed(widget.as_any().type_id(), key),
            None => Self::Indexed(widget.as_any().type_id(), index),
        }
    }
}
