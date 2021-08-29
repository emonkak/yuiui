use std::any::TypeId;
use std::fmt;
use std::mem;

use crate::support::slot_vec::SlotVec;
use crate::widget::element::{
    create_element_tree, Children, Element, ElementId, ElementTree, Key, Patch,
};
use crate::widget::message::{AnyMessage, MessageSender};
use crate::widget::null::Null;
use crate::widget::{AnyState, PolymophicWidget};

use super::reconciler::{ReconcileResult, Reconciler};

#[derive(Debug)]
pub struct RenderTree<Renderer> {
    tree: ElementTree<Renderer>,
    root_id: ElementId,
    render_states: SlotVec<RenderState<Renderer>>,
    message_sender: MessageSender,
}

#[derive(Debug)]
struct RenderState<Renderer> {
    phase: RenderPhase<Renderer>,
    state: AnyState,
    version: usize,
}

#[derive(Debug)]
enum RenderPhase<Renderer> {
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
    pub fn new(message_sender: MessageSender) -> Self {
        let (tree, root_id) = create_element_tree();
        let mut render_states = SlotVec::new();

        render_states.insert_at(root_id, RenderState::new(Box::new(()), tree.version()));

        Self {
            tree,
            root_id,
            render_states,
            message_sender,
        }
    }

    #[inline]
    pub fn root_id(&self) -> ElementId {
        self.root_id
    }

    pub fn render(&mut self, element: Element<Renderer>) -> Vec<Patch<Renderer>> {
        *self.tree[self.root_id] = Element::new(Null, vec![element], None);

        let mut patches = Vec::new();
        let mut current_id = self.root_id;

        while let Some(next_id) = self.render_step(current_id, self.root_id, &mut patches) {
            current_id = next_id;
        }

        patches
    }

    pub fn update(&mut self, target_id: ElementId, version: usize, message: AnyMessage) -> Vec<Patch<Renderer>> {
        let mut patches = Vec::new();
        let render_state = &mut self.render_states[target_id];

        if render_state.version == version {
            let Element { widget, .. } = &*self.tree[target_id];

            if widget.update(&mut render_state.state, message) {
                let mut current_id = target_id;

                while let Some(next_id) = self.render_step(current_id, target_id, &mut patches) {
                    current_id = next_id;
                }
            }
        }

        patches
    }

    fn render_step(
        &mut self,
        target_id: ElementId,
        initial_id: ElementId,
        patches: &mut Vec<Patch<Renderer>>,
    ) -> Option<ElementId> {
        let Element {
            widget, children, ..
        } = &mut *self.tree[target_id];
        let render_state = &mut self.render_states[target_id];

        let old_status = mem::replace(&mut render_state.phase, RenderPhase::Rendered);
        match old_status {
            RenderPhase::Pending(element) => {
                *widget = element.widget;
                *children = element.children;
            }
            RenderPhase::Skipped => unreachable!("Skipped widget"),
            RenderPhase::Fresh | RenderPhase::Rendered => {}
        }

        let rendered_children = widget.render(
            children,
            &render_state.state,
            target_id,
            render_state.version
        );

        for result in self.reconcile_children(target_id, rendered_children) {
            self.handle_reconcile_result(target_id, result, patches);
        }

        self.next_render_target(target_id, initial_id)
    }

    fn reconcile_children(
        &mut self,
        target_id: ElementId,
        children: Children<Renderer>,
    ) -> Reconciler<TypedKey, ElementId, Element<Renderer>> {
        let mut old_keys: Vec<TypedKey> = Vec::new();
        let mut old_element_ids: Vec<Option<ElementId>> = Vec::new();

        for (index, (child_id, child)) in self.tree.children(target_id).enumerate() {
            let key = TypedKey::new(&*child.widget, index, child.key);
            old_keys.push(key);
            old_element_ids.push(Some(child_id));
        }

        let mut new_keys: Vec<TypedKey> = Vec::with_capacity(children.len());
        let mut new_elements: Vec<Option<Element<Renderer>>> = Vec::with_capacity(children.len());

        for (index, element) in children.iter().enumerate() {
            let key = TypedKey::new(&*element.widget, index, element.key);
            new_keys.push(key);
            new_elements.push(Some(element.clone()));
        }

        Reconciler::new(old_keys, old_element_ids, new_keys, new_elements)
    }

    fn handle_reconcile_result(
        &mut self,
        target_id: ElementId,
        result: ReconcileResult<ElementId, Element<Renderer>>,
        patches: &mut Vec<Patch<Renderer>>,
    ) {
        match result {
            ReconcileResult::New(new_element) => {
                let widget_pod = Element::from(new_element);
                let element_id = self.tree.append_child(target_id, widget_pod.clone());
                self.render_states.insert_at(
                    element_id,
                    RenderState::new(widget_pod.widget.initial_state(), self.tree.version()),
                );
                patches.push(Patch::Append(target_id, widget_pod));
            }
            ReconcileResult::Insertion(ref_id, new_element) => {
                let widget_pod = Element::from(new_element);
                let element_id = self.tree.insert_before(ref_id, widget_pod.clone());
                self.render_states.insert_at(
                    element_id,
                    RenderState::new(widget_pod.widget.initial_state(), self.tree.version()),
                );
                patches.push(Patch::Insert(ref_id, widget_pod));
            }
            ReconcileResult::Update(target_id, new_element) => {
                let Element {
                    widget, children, ..
                } = &mut *self.tree[target_id];
                let state = &self.render_states[target_id].state;
                if widget.should_render(children, state, &new_element.widget, &new_element.children)
                {
                    self.render_states[target_id].phase = RenderPhase::Pending(new_element.clone());
                    patches.push(Patch::Update(target_id, new_element));
                } else {
                    self.render_states[target_id].phase = RenderPhase::Skipped;
                }
            }
            ReconcileResult::UpdateAndPlacement(target_id, ref_id, new_element) => {
                let Element {
                    widget, children, ..
                } = &mut *self.tree[target_id];
                let state = &self.render_states[target_id].state;
                if widget.should_render(children, state, &new_element.widget, &new_element.children)
                {
                    self.render_states[target_id].phase = RenderPhase::Pending(new_element.clone());
                    patches.push(Patch::Update(target_id, new_element));
                } else {
                    self.render_states[target_id].phase = RenderPhase::Skipped;
                }
                self.tree.move_position(target_id).insert_before(ref_id);
                patches.push(Patch::Move(target_id, ref_id));
            }
            ReconcileResult::Deletion(target_id) => {
                let (_, subtree) = self.tree.detach(target_id);

                self.render_states.remove(target_id);

                for (child_id, _) in subtree {
                    self.render_states.remove(child_id);
                }

                patches.push(Patch::Remove(target_id));
            }
        }
    }

    fn next_render_target(&self, target_id: ElementId, initial_id: ElementId) -> Option<ElementId> {
        let mut current_node = &self.tree[target_id];

        if let Some(child_id) = self.tree[target_id].first_child() {
            if !matches!(self.render_states[child_id].phase, RenderPhase::Skipped) {
                return Some(child_id);
            }
            current_node = &self.tree[child_id];
        }

        loop {
            while let Some(sibling_id) = current_node.next_sibling() {
                if !matches!(self.render_states[sibling_id].phase, RenderPhase::Skipped) {
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
            |f, element_id, node| {
                let render_state = &self.render_states[element_id];
                write!(f, "<{}", node.widget.short_type_name())?;
                write!(f, " id=\"{}\"", element_id)?;
                if let Some(key) = node.key {
                    write!(f, " key=\"{}\"", key)?;
                }
                match &render_state.phase {
                    RenderPhase::Fresh => write!(f, " fresh")?,
                    RenderPhase::Pending(_) => write!(f, " pending")?,
                    RenderPhase::Rendered => write!(f, " rendered")?,
                    RenderPhase::Skipped => write!(f, " skip")?,
                }
                write!(f, ">")?;
                Ok(())
            },
            |f, _, node| write!(f, "</{}>", node.widget.short_type_name()),
        )
    }
}

impl<Renderer> RenderState<Renderer> {
    fn new(state: AnyState, version: usize) -> Self {
        Self {
            phase: RenderPhase::Fresh,
            state,
            version,
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
