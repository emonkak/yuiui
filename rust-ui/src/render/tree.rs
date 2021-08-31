use std::any::TypeId;
use std::collections::HashMap;
use std::fmt;
use std::mem;

use crate::support::slot_vec::SlotVec;
use crate::widget::element::{
    create_element_tree, Children, Element, ElementId, ElementTree, Key, Patch,
};
use crate::widget::message::{AnyMessage, Message, MessageQueue, MessageSender};
use crate::widget::null::Null;
use crate::widget::{AnyState, PolymophicWidget};

use super::reconciler::{ReconcileResult, Reconciler};

#[derive(Debug)]
pub struct RenderTree<Renderer> {
    tree: ElementTree<Renderer>,
    root_id: ElementId,
    render_states: SlotVec<RenderState<Renderer>>,
    message_sender: MessageSender,
    event_manager: EventManager,
}

#[derive(Debug)]
struct RenderState<Renderer> {
    phase: RenderPhase<Renderer>,
    state: AnyState,
}

#[derive(Debug)]
struct EventManager {
    event_subscribers: HashMap<TypeId, Vec<ElementId>>,
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

impl<Renderer: 'static> RenderTree<Renderer> {
    pub fn new(message_sender: MessageSender) -> Self {
        let (tree, root_id) = create_element_tree();
        let mut render_states = SlotVec::new();

        render_states.insert_at(root_id, RenderState::new(Box::new(())));

        Self {
            tree,
            root_id,
            render_states,
            message_sender,
            event_manager: EventManager::new(),
        }
    }

    #[inline]
    pub fn root_id(&self) -> ElementId {
        self.root_id
    }

    pub fn render(&mut self, element: Element<Renderer>) -> Vec<Patch<Renderer>> {
        *self.tree[self.root_id] = Element::new(
            Null {
                children: vec![element],
            },
            None,
        );

        let mut patches = Vec::new();
        let mut current_id = self.root_id;

        while let Some(next_id) = self.render_step(current_id, self.root_id, &mut patches) {
            current_id = next_id;
        }

        patches
    }

    pub fn update(&mut self, message: Message, patches: &mut Vec<Patch<Renderer>>) {
        let mut message_queue = MessageQueue::new();
        let mut message = message;

        loop {
            match message {
                Message::Broadcast(payload) => {
                    let subscriber_ids = self.event_manager.get_subscribers(payload.type_id());
                    for subscriber_id in subscriber_ids.collect::<Vec<_>>() {
                        self.send_event(subscriber_id, &payload, patches, &mut message_queue);
                    }
                }
                Message::Send(target_id, payload) => {
                    self.send_event(target_id, &payload, patches, &mut message_queue);
                }
            }

            if let Some(next_message) = message_queue.dequeue() {
                message = next_message;
            } else {
                break;
            }
        }
    }

    fn send_event(
        &mut self,
        target_id: ElementId,
        event: &AnyMessage,
        patches: &mut Vec<Patch<Renderer>>,
        message_queue: &mut MessageQueue,
    ) {
        let render_state = &mut self.render_states[target_id];
        let Element { widget, .. } = &*self.tree[target_id];

        if widget.update(&mut render_state.state, &event, message_queue) {
            let mut current_id = target_id;

            while let Some(next_id) = self.render_step(current_id, target_id, patches) {
                current_id = next_id;
            }
        }
    }

    fn render_step(
        &mut self,
        target_id: ElementId,
        initial_id: ElementId,
        patches: &mut Vec<Patch<Renderer>>,
    ) -> Option<ElementId> {
        let Element { widget, .. } = &mut *self.tree[target_id];
        let render_state = &mut self.render_states[target_id];

        let old_status = mem::replace(&mut render_state.phase, RenderPhase::Rendered);
        match old_status {
            RenderPhase::Pending(element) => {
                *widget = element.widget;
            }
            RenderPhase::Skipped => unreachable!("Skipped widget"),
            RenderPhase::Fresh | RenderPhase::Rendered => {}
        }

        let rendered_children = widget.render(&render_state.state, target_id);

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
            ReconcileResult::New(element) => {
                let element_id = self.tree.append_child(target_id, element.clone());
                self.render_states
                    .insert_at(element_id, RenderState::new(element.widget.initial_state()));
                self.event_manager.add_subscriber(element_id, &element);
                patches.push(Patch::Append(target_id, element));
            }
            ReconcileResult::Insertion(ref_id, element) => {
                let element_id = self.tree.insert_before(ref_id, element.clone());
                self.render_states
                    .insert_at(element_id, RenderState::new(element.widget.initial_state()));
                self.event_manager.add_subscriber(element_id, &element);
                patches.push(Patch::Insert(ref_id, element));
            }
            ReconcileResult::Update(target_id, new_element) => {
                let Element { widget, .. } = &mut *self.tree[target_id];
                if widget.should_render(new_element.widget.as_any()) {
                    self.render_states[target_id].phase = RenderPhase::Pending(new_element.clone());
                    patches.push(Patch::Update(target_id, new_element));
                } else {
                    self.render_states[target_id].phase = RenderPhase::Skipped;
                }
            }
            ReconcileResult::UpdateAndPlacement(target_id, ref_id, new_element) => {
                let Element { widget, .. } = &mut *self.tree[target_id];
                if widget.should_render(new_element.widget.as_any()) {
                    self.render_states[target_id].phase = RenderPhase::Pending(new_element.clone());
                    patches.push(Patch::Update(target_id, new_element));
                } else {
                    self.render_states[target_id].phase = RenderPhase::Skipped;
                }
                self.tree.move_position(target_id).insert_before(ref_id);
                patches.push(Patch::Move(target_id, ref_id));
            }
            ReconcileResult::Deletion(target_id) => {
                let (node, subtree) = self.tree.detach(target_id);

                self.render_states.remove(target_id);
                self.event_manager.remove_subscriber(target_id, &*node);

                for (child_id, child) in subtree {
                    self.render_states.remove(child_id);
                    self.event_manager.remove_subscriber(target_id, &*child);
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
    fn new(state: AnyState) -> Self {
        Self {
            phase: RenderPhase::Fresh,
            state,
        }
    }
}

impl EventManager {
    fn new() -> Self {
        Self {
            event_subscribers: HashMap::new(),
        }
    }

    fn get_subscribers(&self, type_id: TypeId) -> impl Iterator<Item = ElementId> + '_ {
        self.event_subscribers
            .get(&type_id)
            .map_or(&[] as &[ElementId], |element_ids| element_ids.as_slice())
            .iter()
            .copied()
    }

    fn add_subscriber<Renderer>(&mut self, element_id: ElementId, element: &Element<Renderer>) {
        self.event_subscribers
            .entry(element.widget.inbound_type())
            .or_default()
            .push(element_id);
    }

    fn remove_subscriber<Renderer>(&mut self, element_id: ElementId, element: &Element<Renderer>) {
        let found_buckets = self
            .event_subscribers
            .get_mut(&element.widget.inbound_type());
        if let Some(buckets) = found_buckets {
            let found_index = buckets.iter().position(|id| *id == element_id);
            if let Some(index) = found_index {
                buckets.remove(index);
            }
        }
    }
}

impl TypedKey {
    fn new<Renderer: 'static>(
        widget: &(dyn PolymophicWidget<Renderer> + 'static),
        index: usize,
        key: Option<Key>,
    ) -> Self {
        match key {
            Some(key) => Self::Keyed(widget.as_any().type_id(), key),
            None => Self::Indexed(widget.as_any().type_id(), index),
        }
    }
}
