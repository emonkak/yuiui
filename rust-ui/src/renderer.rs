use std::any::TypeId;
use std::marker::PhantomData;

use crate::event::EventType;
use crate::event::handler::{EventContext, WidgetHandler};
use crate::reconciler::{ReconcileResult, Reconciler};
use crate::tree::{NodeId, Tree};
use crate::widget::PolymophicWidget;
use crate::widget::element::{Children, Element, Key};
use crate::widget::tree::{WidgetFlag, WidgetPod, WidgetTree};

#[derive(Clone, Debug)]
pub struct RenderTree<Handle> {
    pub tree: WidgetTree<Handle>,
    pub root_id: NodeId,
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

impl<Handle> RenderTree<Handle> {
    pub fn render(element: Element<Handle>) -> Self {
        let mut tree = Tree::new();
        let root_id = tree.attach(WidgetPod::from(element));

        let mut current_id = root_id;
        while let Some(next_id) = render_step(&mut tree, current_id, root_id) {
            current_id = next_id;
        }

        Self {
            tree,
            root_id,
        }
    }

    pub fn update(&mut self, target_id: NodeId) {
        let mut current_id = target_id;

        while let Some(next_id) = render_step(&mut self.tree, current_id, self.root_id) {
            current_id = next_id;
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
}

fn render_step<Handle>(
    tree: &mut WidgetTree<Handle>,
    target_id: NodeId,
    root_id: NodeId,
) -> Option<NodeId> {
    let WidgetPod {
        widget,
        children,
        state,
        ..
    } = &*tree[target_id];
    let rendered_children =
        widget.render(children.clone(), &mut **state.lock().unwrap(), target_id);
    reconcile_children(tree, target_id, rendered_children);
    next_render_step(&tree, target_id, root_id)
}

fn next_render_step<Handle>(
    tree: &WidgetTree<Handle>,
    target_id: NodeId,
    root_id: NodeId,
) -> Option<NodeId> {
    let mut current_node = &tree[target_id];

    if current_node.flags.contains([WidgetFlag::Fresh, WidgetFlag::Dirty]) {
        if let Some(first_child) = tree[target_id].first_child() {
            return Some(first_child);
        }
    }

    loop {
        while let Some(sibling_id) = current_node.next_sibling() {
            current_node = &tree[sibling_id];
            if current_node.flags.contains([WidgetFlag::Fresh, WidgetFlag::Dirty]) {
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

fn reconcile_children<Handle>(
    tree: &mut WidgetTree<Handle>,
    target_id: NodeId,
    children: Children<Handle>,
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
        handle_reconcile_result(tree, target_id, result);
    }
}

fn handle_reconcile_result<Handle>(
    tree: &mut WidgetTree<Handle>,
    target_id: NodeId,
    result: ReconcileResult<NodeId, Element<Handle>>,
) {
    match result {
        ReconcileResult::Create(new_element) => {
            tree.append_child(target_id, WidgetPod::from(new_element));
        }
        ReconcileResult::CreateAndPlacement(ref_id, new_element) => {
            tree.insert_before(ref_id, WidgetPod::from(new_element));
        }
        ReconcileResult::Update(target_id, new_element) => {
            mark_as_dirty(tree, target_id, new_element);
        }
        ReconcileResult::UpdateAndPlacement(target_id, ref_id, new_element) => {
            mark_as_dirty(tree, target_id, new_element);
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

fn mark_as_dirty<Handle>(
    tree: &mut WidgetTree<Handle>,
    target_id: NodeId,
    element: Element<Handle>,
) {
    let WidgetPod {
        widget,
        state,
        children,
        deleted_children,
        flags,
        key,
        ..
    } = &mut *tree[target_id];

    *children = element.children;
    *deleted_children = Vec::new();
    *key = element.key;

    if widget.should_update(&*element.widget, &**state.lock().unwrap()) {
        *widget = element.widget;
        *flags = WidgetFlag::Dirty.into();

        for (_, parent) in tree.ancestors_mut(target_id) {
            if parent.flags.contains(WidgetFlag::Dirty) {
                break;
            }
            parent.flags |= WidgetFlag::Dirty;
        }
    } else {
        *flags = WidgetFlag::None.into();
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
