use std::any::TypeId;
use std::marker::PhantomData;

use crate::event::handler::{GlobalHandler, WidgetHandler};
use crate::event::{EventContext, EventType};
use crate::reconciler::{ReconcileResult, Reconciler};
use crate::slot_vec::SlotVec;
use crate::tree::{NodeId, Tree};
use crate::widget::element::{Children, Element, Key};
use crate::widget::{PolymophicWidget, WidgetPod, WidgetTree};

#[derive(Debug)]
pub struct Renderer<Handle> {
    render_states: SlotVec<RenderState<Handle>>,
}

#[derive(Debug)]
pub struct RenderState<Handle> {
    pub children: Option<Children<Handle>>,
    pub deleted_children: Vec<(NodeId, WidgetPod<Handle>)>,
    pub key: Option<Key>,
    pub mounted: bool,
}

pub struct RenderContext<Widget: ?Sized, Handle, State> {
    node_id: NodeId,
    _widget: PhantomData<Widget>,
    _handle: PhantomData<Handle>,
    _state: PhantomData<State>,
}

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
enum TypedKey {
    Keyed(TypeId, Key),
    Indexed(TypeId, usize),
}

impl<Handle> Renderer<Handle> {
    pub fn new() -> Self {
        Self {
            render_states: SlotVec::new(),
        }
    }

    pub fn render(&mut self, element: Element<Handle>) -> (NodeId, WidgetTree<Handle>) {
        let mut tree = Tree::new();
        let root_id = tree.attach(WidgetPod::new(element.widget));

        self.render_states
            .insert_at(root_id, RenderState::new(Vec::new(), None));

        let mut current_id = root_id;
        while let Some(next_id) = self.render_step(current_id, root_id, &mut tree) {
            current_id = next_id;
        }

        (root_id, tree)
    }

    pub fn update(
        &mut self,
        element: Element<Handle>,
        root_id: NodeId,
        tree: &mut WidgetTree<Handle>,
    ) {
        self.update_render_state(root_id, element, tree);

        let mut current_id = root_id;

        while let Some(next_id) = self.render_step(current_id, root_id, tree) {
            current_id = next_id;
        }
    }

    fn render_step(
        &mut self,
        node_id: NodeId,
        root_id: NodeId,
        tree: &mut WidgetTree<Handle>,
    ) -> Option<NodeId> {
        let WidgetPod { widget, state, .. } = &*tree[node_id];
        let render_state = &mut self.render_states[node_id];

        if let Some(children) = render_state.children.take() {
            let rendered_children = widget.render(children, &mut **state.lock().unwrap(), node_id);
            self.reconcile_children(node_id, rendered_children.into(), tree);
        }

        self.next_render_step(node_id, root_id, &tree)
    }

    fn next_render_step(
        &self,
        node_id: NodeId,
        root_id: NodeId,
        tree: &WidgetTree<Handle>,
    ) -> Option<NodeId> {
        if let Some(first_child) = tree[node_id].first_child() {
            return Some(first_child);
        }

        let mut currnet_node_id = node_id;

        loop {
            let current_node = &tree[currnet_node_id];
            if let Some(sibling_id) = current_node.next_sibling() {
                return Some(sibling_id);
            }

            if let Some(parent_id) = current_node
                .parent()
                .filter(|&parent_id| parent_id != root_id)
            {
                currnet_node_id = parent_id;
            } else {
                break;
            }
        }

        None
    }

    fn reconcile_children(
        &mut self,
        target_id: NodeId,
        children: Children<Handle>,
        tree: &mut WidgetTree<Handle>,
    ) {
        let mut old_keys: Vec<TypedKey> = Vec::new();
        let mut old_node_ids: Vec<Option<NodeId>> = Vec::new();

        for (index, (child_id, child)) in tree.children(target_id).enumerate() {
            let child_render_state = &self.render_states[child_id];
            let key = key_of(&*child.widget, index, child_render_state.key);
            old_keys.push(key);
            old_node_ids.push(Some(child_id));
        }

        let mut new_keys: Vec<TypedKey> = Vec::with_capacity(children.len());
        let mut new_elements: Vec<Option<Element<Handle>>> = Vec::with_capacity(children.len());

        for (index, element) in children.into_iter().enumerate() {
            let key = key_of(&*element.widget, index, element.key);
            new_keys.push(key);
            new_elements.push(Some(element));
        }

        let reconciler =
            Reconciler::new(&old_keys, &mut old_node_ids, &new_keys, &mut new_elements);

        for result in reconciler {
            self.handle_reconcile_result(target_id, result, tree);
        }
    }

    fn handle_reconcile_result(
        &mut self,
        target_id: NodeId,
        result: ReconcileResult<NodeId, Element<Handle>>,
        tree: &mut WidgetTree<Handle>,
    ) {
        match result {
            ReconcileResult::New(new_element) => {
                let node_id = tree.next_node_id();
                let render_state = RenderState::new(new_element.children, new_element.key);
                tree.append_child(target_id, WidgetPod::new(new_element.widget));
                self.render_states.insert_at(node_id, render_state);
            }
            ReconcileResult::NewPlacement(ref_id, new_element) => {
                let node_id = tree.next_node_id();
                let render_state = RenderState::new(new_element.children, new_element.key);
                tree.insert_before(ref_id, WidgetPod::new(new_element.widget));
                self.render_states.insert_at(node_id, render_state);
            }
            ReconcileResult::Update(target_id, new_element) => {
                self.update_render_state(target_id, new_element, tree);
            }
            ReconcileResult::UpdatePlacement(target_id, ref_id, new_element) => {
                self.update_render_state(target_id, new_element, tree);
                tree.move_position(target_id).insert_before(ref_id);
            }
            ReconcileResult::Deletion(target_id) => {
                let mut deleted_children = Vec::new();

                for (node_id, node) in tree.detach_subtree(target_id) {
                    deleted_children.push((node_id, node.into_inner()));
                }

                let parent = tree[target_id].parent();

                if let Some(parent_id) = parent {
                    self.render_states[parent_id].deleted_children = deleted_children;
                }
            }
        }
    }

    fn update_render_state(
        &mut self,
        node_id: NodeId,
        element: Element<Handle>,
        tree: &mut WidgetTree<Handle>,
    ) {
        let WidgetPod {
            widget,
            state,
            dirty,
        } = &mut *tree[node_id];

        if widget.should_update(&*element.widget, &**state.lock().unwrap()) {
            let render_state = &mut self.render_states[node_id];
            render_state.children = Some(element.children);
            render_state.key = element.key;

            *dirty = true;

            for (_, parent) in tree.ancestors_mut(node_id) {
                if parent.dirty {
                    break;
                }
                parent.dirty = true;
            }
        } else {
            *dirty = false;
        }
    }
}

impl<Handle> RenderState<Handle> {
    pub fn new(children: Children<Handle>, key: Option<Key>) -> Self {
        Self {
            children: Some(children),
            deleted_children: Vec::new(),
            key,
            mounted: false,
        }
    }
}

impl<Widget, Handle, State> RenderContext<Widget, Handle, State>
where
    Widget: 'static,
    State: 'static,
{
    pub fn new(node_id: NodeId) -> Self {
        Self {
            node_id: node_id,
            _widget: PhantomData,
            _handle: PhantomData,
            _state: PhantomData,
        }
    }

    pub fn use_handler<EventType>(
        &self,
        event_type: EventType,
        callback: fn(&Widget, &EventType::Event, &mut State, &mut EventContext),
    ) -> WidgetHandler<EventType, EventType::Event, Widget, State>
    where
        EventType: self::EventType + 'static,
    {
        WidgetHandler::new(event_type, callback, self.node_id)
    }

    pub fn use_global_handler<EventType>(
        &self,
        event_type: EventType,
        callback: fn(&EventType::Event, &mut EventContext),
    ) -> GlobalHandler<EventType, EventType::Event>
    where
        EventType: self::EventType + 'static,
    {
        GlobalHandler::new(event_type, callback)
    }
}

fn key_of<Handle>(
    widget: &dyn PolymophicWidget<Handle>,
    index: usize,
    key: Option<Key>,
) -> TypedKey {
    match key {
        Some(key) => TypedKey::Keyed(widget.as_any().type_id(), key),
        None => TypedKey::Indexed(widget.as_any().type_id(), index),
    }
}
