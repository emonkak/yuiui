use std::any::TypeId;
use std::marker::PhantomData;

use crate::event::EventType;
use crate::event::handler::{EventContext, WidgetHandler};
use crate::reconciler::{ReconcileResult, Reconciler};
use crate::tree::{NodeId, Tree};
use crate::widget::element::{Children, Element, Key};
use crate::widget::{PolymophicWidget, WidgetPod, WidgetTree};

#[derive(Debug)]
pub struct Renderer<Handle> {
    _handle: PhantomData<Handle>,
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
            _handle: PhantomData,
        }
    }

    pub fn render(&mut self, element: Element<Handle>) -> (NodeId, WidgetTree<Handle>) {
        let mut tree = Tree::new();
        let root_id = tree.attach(WidgetPod::from(element));

        let mut current_id = root_id;
        while let Some(next_id) = self.render_step(current_id, root_id, &mut tree) {
            current_id = next_id;
        }

        (root_id, tree)
    }

    pub fn update(&mut self, root_id: NodeId, tree: &mut WidgetTree<Handle>) {
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
        let WidgetPod {
            widget,
            children,
            state,
            ..
        } = &*tree[node_id];
        let rendered_children =
            widget.render(children.clone(), &mut **state.lock().unwrap(), node_id);
        self.reconcile_children(node_id, rendered_children, tree);
        self.next_render_step(node_id, root_id, &tree)
    }

    fn next_render_step(
        &self,
        node_id: NodeId,
        root_id: NodeId,
        tree: &WidgetTree<Handle>,
    ) -> Option<NodeId> {
        let mut current_node = &tree[node_id];

        if current_node.dirty {
            if let Some(first_child) = tree[node_id].first_child() {
                return Some(first_child);
            }
        }

        loop {
            while let Some(sibling_id) = current_node.next_sibling() {
                current_node = &tree[sibling_id];
                if current_node.dirty {
                    return Some(sibling_id);
                }
            }

            if let Some(parent_id) = current_node
                .parent()
                .filter(|&parent_id| parent_id != root_id)
            {
                current_node = &tree[parent_id];
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
            let key = key_of(&*child.widget, index, child.key);
            old_keys.push(key);
            old_node_ids.push(Some(child_id));
        }

        let mut new_keys: Vec<TypedKey> = Vec::with_capacity(children.len());
        let mut new_elements: Vec<Option<Element<Handle>>> = Vec::with_capacity(children.len());

        for (index, element) in children.iter().enumerate() {
            let key = key_of(&*element.widget, index, element.key);
            new_keys.push(key);
            new_elements.push(Some(element.clone()));
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
            ReconcileResult::Create(new_element) => {
                tree.append_child(target_id, WidgetPod::from(new_element));
            }
            ReconcileResult::CreateAndPlacement(ref_id, new_element) => {
                tree.insert_before(ref_id, WidgetPod::from(new_element));
            }
            ReconcileResult::Update(target_id, new_element) => {
                self.mark_as_dirty(target_id, new_element, tree);
            }
            ReconcileResult::UpdateAndPlacement(target_id, ref_id, new_element) => {
                self.mark_as_dirty(target_id, new_element, tree);
                tree.move_position(target_id).insert_before(ref_id);
            }
            ReconcileResult::Delete(target_id) => {
                let (_, detached_node) = tree.detach_subtree(target_id).last().unwrap();

                if let Some(parent_id) = detached_node.parent() {
                    let WidgetPod { deleted_children, ..  } = &mut *tree[parent_id];
                    deleted_children.push(target_id);
                }
            }
        }
    }

    fn mark_as_dirty(
        &mut self,
        node_id: NodeId,
        element: Element<Handle>,
        tree: &mut WidgetTree<Handle>,
    ) {
        let WidgetPod {
            widget,
            state,
            children,
            deleted_children,
            dirty,
            key,
            ..
        } = &mut *tree[node_id];

        *children = element.children;
        *deleted_children = Vec::new();
        *key = element.key;

        if widget.should_update(&*element.widget, &**state.lock().unwrap()) {
            *widget = element.widget;
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
        WidgetHandler::new(event_type, self.node_id, callback)
    }

    // pub fn use_global_handler<EventType>(
    //     &self,
    //     event_type: EventType,
    //     callback: fn(&EventType::Event, &mut EventContext),
    // ) -> GlobalHandler<EventType, EventType::Event>
    // where
    //     EventType: self::EventType + 'static,
    // {
    //     GlobalHandler::new(event_type, callback)
    // }
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
