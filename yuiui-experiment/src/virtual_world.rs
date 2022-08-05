use std::any::Any;
use std::pin::Pin;
use std::ptr::NonNull;
use yuiui_support::slot_tree::{NodeId, SlotTree};

use crate::context::Context;
use crate::element::Element;
use crate::element_seq::ElementSeq as _;
use crate::real_world::RealWorld;
use crate::view::{AnyView, View, ViewInspector, ViewPod};

pub struct VirtualWorld<E: Element> {
    view_pod: Pin<Box<ViewPod<E::View, E::Components>>>,
    tree: VirtualTree,
    context: Context,
}

impl<E: Element> VirtualWorld<E> {
    pub fn new(element: E) -> Self {
        let mut context = Context::new();
        let view_pod = Box::pin(element.build(&mut context));
        let tree = VirtualTree::new(&*view_pod.as_ref());
        println!("{:#?}", tree.arena);
        println!("{:#?}", context);
        Self { view_pod, tree, context }
    }

    pub fn realize(&self) -> RealWorld<E> {
        let widget_pod = E::compile(&self.view_pod());
        RealWorld::new(widget_pod)
    }

    pub fn view_pod(&self) -> &ViewPod<E::View, E::Components> {
        Pin::get_ref(self.view_pod.as_ref())
    }
}

struct VirtualTree {
    arena: SlotTree<VirtualNode>,
}

impl VirtualTree {
    fn new<V: View, C>(view_pod: &ViewPod<V, C>) -> Self {
        let arena = SlotTree::new(VirtualNode::new(&*view_pod));
        let mut tree = VirtualTree { arena };
        V::Children::inspect(&mut tree, NodeId::ROOT, &view_pod.children);
        tree
    }
}

impl ViewInspector for VirtualTree {
    type Id = NodeId;

    fn push<V: View, C>(&mut self, origin: Self::Id, view_pod: &ViewPod<V, C>) -> Self::Id {
        self.arena
            .cursor_mut(origin)
            .append_child(VirtualNode::new(view_pod))
    }
}

#[derive(Debug)]
struct VirtualNode {
    view: NonNull<dyn AnyView>,
    children: NonNull<dyn Any>,
}

impl VirtualNode {
    fn new<V: View, C>(view_pod: &ViewPod<V, C>) -> Self {
        unsafe {
            Self {
                view: NonNull::new_unchecked(&view_pod.view as *const dyn AnyView as *mut _),
                children: NonNull::new_unchecked(&view_pod.children as *const dyn Any as *mut _),
            }
        }
    }
}
