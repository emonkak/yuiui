use crate::context::Context;
use crate::element::Element;
use crate::node::{UINode, VNode};
use crate::view::View;

pub trait ElementSeq: 'static {
    type VNodes;

    type UINodes;

    fn render(v_nodes: &Self::VNodes) -> Self::UINodes;

    fn rerender(v_nodes: &Self::VNodes, ui_nodes: &mut Self::UINodes) -> bool;

    fn invalidate(v_nodes: &Self::VNodes, context: &mut Context);

    fn build(self, context: &mut Context) -> Self::VNodes;

    fn rebuild(self, v_nodes: &mut Self::VNodes, context: &mut Context) -> bool;
}

impl ElementSeq for () {
    type VNodes = ();

    type UINodes = ();

    fn render(_v_nodes: &Self::VNodes) -> Self::UINodes {
        ()
    }

    fn rerender(_v_nodes: &Self::VNodes, _widgets: &mut Self::UINodes) -> bool {
        false
    }

    fn invalidate(_v_nodes: &Self::VNodes, _context: &mut Context) {}

    fn build(self, _context: &mut Context) -> Self::VNodes {
        ()
    }

    fn rebuild(self, _v_nodes: &mut Self::VNodes, _context: &mut Context) -> bool {
        false
    }
}

impl<T1> ElementSeq for (T1,)
where
    T1: Element,
{
    type VNodes = (VNode<T1::View, T1::Components>,);

    type UINodes = (UINode<<T1::View as View>::Widget>,);

    fn render(v_nodes: &Self::VNodes) -> Self::UINodes {
        (v_nodes.0.build(),)
    }

    fn rerender(v_nodes: &Self::VNodes, ui_nodes: &mut Self::UINodes) -> bool {
        v_nodes.0.rebuild(&mut ui_nodes.0.widget, &mut ui_nodes.0.children)
    }

    fn invalidate(v_nodes: &Self::VNodes, context: &mut Context) {
        v_nodes.0.invalidate(context);
    }

    fn build(self, context: &mut Context) -> Self::VNodes {
        (self.0.build(context),)
    }

    fn rebuild(self, v_nodes: &mut Self::VNodes, context: &mut Context) -> bool {
        self.0.rebuild(
            &mut v_nodes.0.view,
            &mut v_nodes.0.children,
            &mut v_nodes.0.components,
            context,
        )
    }
}

impl<T1, T2> ElementSeq for (T1, T2)
where
    T1: Element,
    T2: Element,
{
    type VNodes = (
        VNode<T1::View, T1::Components>,
        VNode<T2::View, T2::Components>,
    );

    type UINodes = (
        UINode<<T1::View as View>::Widget>,
        UINode<<T2::View as View>::Widget>,
    );

    fn render(v_nodes: &Self::VNodes) -> Self::UINodes {
        (v_nodes.0.build(), v_nodes.1.build())
    }

    fn rerender(v_nodes: &Self::VNodes, ui_nodes: &mut Self::UINodes) -> bool {
        let mut has_changed = false;
        has_changed |= &v_nodes.0.rebuild(&mut ui_nodes.0.widget, &mut ui_nodes.0.children);
        has_changed |= &v_nodes.1.rebuild(&mut ui_nodes.1.widget, &mut ui_nodes.1.children);
        has_changed
    }

    fn invalidate(v_nodes: &Self::VNodes, context: &mut Context) {
        v_nodes.0.invalidate(context);
        v_nodes.1.invalidate(context);
    }

    fn build(self, context: &mut Context) -> Self::VNodes {
        (self.0.build(context), self.1.build(context))
    }

    fn rebuild(self, v_nodes: &mut Self::VNodes, context: &mut Context) -> bool {
        let mut has_changed = false;
        has_changed |= self.0.rebuild(
            &mut v_nodes.0.view,
            &mut v_nodes.0.children,
            &mut v_nodes.0.components,
            context,
        );
        has_changed |= self.1.rebuild(
            &mut v_nodes.1.view,
            &mut v_nodes.1.children,
            &mut v_nodes.1.components,
            context,
        );
        has_changed
    }
}

impl<T1, T2, T3> ElementSeq for (T1, T2, T3)
where
    T1: Element,
    T2: Element,
    T3: Element,
{
    type VNodes = (
        VNode<T1::View, T1::Components>,
        VNode<T2::View, T2::Components>,
        VNode<T3::View, T3::Components>,
    );

    type UINodes = (
        UINode<<T1::View as View>::Widget>,
        UINode<<T2::View as View>::Widget>,
        UINode<<T3::View as View>::Widget>,
    );

    fn render(v_nodes: &Self::VNodes) -> Self::UINodes {
        (
            v_nodes.0.build(),
            v_nodes.1.build(),
            v_nodes.2.build(),
        )
    }

    fn rerender(v_nodes: &Self::VNodes, ui_nodes: &mut Self::UINodes) -> bool {
        let mut has_changed = false;
        has_changed |= &v_nodes.0.rebuild(&mut ui_nodes.0.widget, &mut ui_nodes.0.children);
        has_changed |= &v_nodes.1.rebuild(&mut ui_nodes.1.widget, &mut ui_nodes.1.children);
        has_changed |= &v_nodes.2.rebuild(&mut ui_nodes.2.widget, &mut ui_nodes.2.children);
        has_changed
    }

    fn invalidate(v_nodes: &Self::VNodes, context: &mut Context) {
        v_nodes.0.invalidate(context);
        v_nodes.1.invalidate(context);
        v_nodes.2.invalidate(context);
    }

    fn build(self, context: &mut Context) -> Self::VNodes {
        (
            self.0.build(context),
            self.1.build(context),
            self.2.build(context),
        )
    }

    fn rebuild(self, v_nodes: &mut Self::VNodes, context: &mut Context) -> bool {
        let mut has_changed = false;
        has_changed |= self.0.rebuild(
            &mut v_nodes.0.view,
            &mut v_nodes.0.children,
            &mut v_nodes.0.components,
            context,
        );
        has_changed |= self.1.rebuild(
            &mut v_nodes.1.view,
            &mut v_nodes.1.children,
            &mut v_nodes.1.components,
            context,
        );
        has_changed |= self.2.rebuild(
            &mut v_nodes.2.view,
            &mut v_nodes.2.children,
            &mut v_nodes.2.components,
            context,
        );
        has_changed
    }
}

impl<T> ElementSeq for Option<T>
where
    T: Element,
{
    type VNodes = Option<VNode<T::View, T::Components>>;

    type UINodes = Option<UINode<<T::View as View>::Widget>>;

    fn render(v_nodes: &Self::VNodes) -> Self::UINodes {
        v_nodes.as_ref().map(VNode::build)
    }

    fn rerender(v_nodes: &Self::VNodes, ui_nodes: &mut Self::UINodes) -> bool {
        match (v_nodes, ui_nodes.as_mut()) {
            (Some(v_node), Some(ui_node)) => {
                v_node.rebuild(&mut ui_node.widget, &mut ui_node.children)
            }
            (Some(v_node), None) => {
                *ui_nodes = Some(v_node.build());
                true
            }
            (None, Some(_)) => {
                *ui_nodes = None;
                true
            }
            (None, None) => false,
        }
    }

    fn invalidate(v_nodes: &Self::VNodes, context: &mut Context) {
        if let Some(v_node) = v_nodes {
            v_node.invalidate(context);
        }
    }

    fn build(self, context: &mut Context) -> Self::VNodes {
        self.map(|element| element.build(context))
    }

    fn rebuild(self, v_nodes: &mut Self::VNodes, context: &mut Context) -> bool {
        match (self, v_nodes.as_mut()) {
            (Some(element), Some(v_node)) => element.rebuild(
                &mut v_node.view,
                &mut v_node.children,
                &mut v_node.components,
                context,
            ),
            (Some(element), None) => {
                *v_nodes = Some(element.build(context));
                true
            }
            (None, Some(v_node)) => {
                v_node.invalidate(context);
                *v_nodes = None;
                true
            }
            (None, None) => false,
        }
    }
}

impl<T> ElementSeq for Vec<T>
where
    T: Element,
{
    type VNodes = Vec<VNode<T::View, T::Components>>;

    type UINodes = Vec<UINode<<T::View as View>::Widget>>;

    fn render(v_nodes: &Self::VNodes) -> Self::UINodes {
        v_nodes.into_iter().map(VNode::build).collect()
    }

    fn rerender(v_nodes: &Self::VNodes, ui_nodes: &mut Self::UINodes) -> bool {
        if v_nodes.len() < ui_nodes.len() {
            ui_nodes.drain(ui_nodes.len() - v_nodes.len() - 1..);
        } else {
            ui_nodes.reserve_exact(v_nodes.len());
        }

        let reuse_len = v_nodes.len().min(ui_nodes.len());
        let mut has_changed = false;

        for (i, v_node) in v_nodes.into_iter().enumerate() {
            if i < reuse_len {
                let ui_node = &mut ui_nodes[i];
                if v_node.rebuild(&mut ui_node.widget, &mut ui_node.children) {
                    has_changed = true;
                }
            } else {
                let ui_node = v_node.build();
                ui_nodes.push(ui_node);
                has_changed = true;
            }
        }

        has_changed
    }

    fn invalidate(v_nodes: &Self::VNodes, context: &mut Context) {
        for v_node in v_nodes {
            v_node.invalidate(context);
        }
    }

    fn build(self, context: &mut Context) -> Self::VNodes {
        self.into_iter()
            .map(|element| element.build(context))
            .collect()
    }

    fn rebuild(self, v_nodes: &mut Self::VNodes, context: &mut Context) -> bool {
        if self.len() < v_nodes.len() {
            for v_node in v_nodes.drain(v_nodes.len() - self.len() - 1..) {
                v_node.invalidate(context);
            }
        } else {
            v_nodes.reserve_exact(self.len());
        }

        let reuse_len = self.len().min(v_nodes.len());
        let mut has_changed = false;

        for (i, element) in self.into_iter().enumerate() {
            if i < reuse_len {
                let v_node = &mut v_nodes[i];
                if element.rebuild(
                    &mut v_node.view,
                    &mut v_node.children,
                    &mut v_node.components,
                    context,
                ) {
                    has_changed = true;
                }
            } else {
                let v_node = element.build(context);
                v_nodes.push(v_node);
                has_changed = true;
            }
        }

        has_changed
    }
}

#[derive(Debug, Clone)]
pub enum Either<L, R> {
    Left(L),
    Right(R),
}

impl<L, R> Either<L, R> {
    pub fn as_ref(&self) -> Either<&L, &R> {
        match self {
            Either::Left(value) => Either::Left(value),
            Either::Right(value) => Either::Right(value),
        }
    }

    pub fn as_mut(&mut self) -> Either<&mut L, &mut R> {
        match self {
            Either::Left(value) => Either::Left(value),
            Either::Right(value) => Either::Right(value),
        }
    }
}

impl<L, R> ElementSeq for Either<L, R>
where
    L: Element,
    R: Element,
{
    type VNodes = Either<VNode<L::View, L::Components>, VNode<R::View, R::Components>>;

    type UINodes = Either<UINode<<L::View as View>::Widget>, UINode<<R::View as View>::Widget>>;

    fn render(v_nodes: &Self::VNodes) -> Self::UINodes {
        match v_nodes {
            Either::Left(v_node) => Either::Left(v_node.build()),
            Either::Right(v_node) => Either::Right(v_node.build()),
        }
    }

    fn rerender(v_nodes: &Self::VNodes, ui_nodes: &mut Self::UINodes) -> bool {
        match (v_nodes, ui_nodes.as_mut()) {
            (Either::Left(v_node), Either::Left(ui_node)) => {
                v_node.rebuild(&mut ui_node.widget, &mut ui_node.children)
            }
            (Either::Right(v_node), Either::Right(ui_node)) => {
                v_node.rebuild(&mut ui_node.widget, &mut ui_node.children)
            }
            (Either::Left(v_node), Either::Right(_)) => {
                *ui_nodes = Either::Left(v_node.build());
                true
            }
            (Either::Right(v_node), Either::Left(_)) => {
                *ui_nodes = Either::Right(v_node.build());
                true
            }
        }
    }

    fn invalidate(v_nodes: &Self::VNodes, context: &mut Context) {
        match v_nodes {
            Either::Left(v_node) => v_node.invalidate(context),
            Either::Right(v_node) => v_node.invalidate(context),
        }
    }

    fn build(self, context: &mut Context) -> Self::VNodes {
        match self {
            Either::Left(element) => Either::Left(element.build(context)),
            Either::Right(element) => Either::Right(element.build(context)),
        }
    }

    fn rebuild(self, v_nodes: &mut Self::VNodes, context: &mut Context) -> bool {
        match (self, v_nodes.as_mut()) {
            (Either::Left(element), Either::Left(v_node)) => element.rebuild(
                &mut v_node.view,
                &mut v_node.children,
                &mut v_node.components,
                context,
            ),
            (Either::Right(element), Either::Right(v_node)) => element.rebuild(
                &mut v_node.view,
                &mut v_node.children,
                &mut v_node.components,
                context,
            ),
            (Either::Left(element), Either::Right(v_node)) => {
                v_node.invalidate(context);
                *v_nodes = Either::Left(element.build(context));
                true
            }
            (Either::Right(element), Either::Left(v_node)) => {
                v_node.invalidate(context);
                *v_nodes = Either::Right(element.build(context));
                true
            }
        }
    }
}
