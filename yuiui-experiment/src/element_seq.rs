use crate::element::Element;
use crate::view::{View, ViewPod};
use crate::widget::WidgetPod;
use crate::context::Context;

pub trait ElementSeq: 'static {
    type Views;

    type Widgets;

    fn depth() -> usize;

    fn compile(views: &Self::Views) -> Self::Widgets;

    fn recompile(views: &Self::Views, widgets: &mut Self::Widgets) -> bool;

    fn invalidate(views: &Self::Views, context: &mut Context);

    fn build(self, context: &mut Context) -> Self::Views;

    fn rebuild(self, views: &mut Self::Views, context: &mut Context) -> bool;
}

impl ElementSeq for () {
    type Views = ();

    type Widgets = ();

    fn depth() -> usize {
        0
    }

    fn compile(_views: &Self::Views) -> Self::Widgets {
        ()
    }

    fn recompile(_views: &Self::Views, _widgets: &mut Self::Widgets) -> bool {
        false
    }

    fn invalidate(_views: &Self::Views, _context: &mut Context) {
    }

    fn build(self, _context: &mut Context) -> Self::Views {
        ()
    }

    fn rebuild(self, _views: &mut Self::Views, _context: &mut Context) -> bool {
        false
    }
}

impl<T1> ElementSeq for (T1,)
where
    T1: Element,
{
    type Views = (ViewPod<T1::View, T1::Components>,);

    type Widgets = (WidgetPod<<T1::View as View>::Widget>,);

    fn depth() -> usize {
        T1::View::depth()
    }

    fn compile(views: &Self::Views) -> Self::Widgets {
        (T1::compile(&views.0),)
    }

    fn recompile(views: &Self::Views, widgets: &mut Self::Widgets) -> bool {
        T1::recompile(&views.0, &mut widgets.0.widget, &mut widgets.0.children)
    }

    fn invalidate(views: &Self::Views, context: &mut Context) {
        T1::invalidate(&views.0, context);
    }

    fn build(self, context: &mut Context) -> Self::Views {
        (self.0.build(context),)
    }

    fn rebuild(self, views: &mut Self::Views, context: &mut Context) -> bool {
        self.0.rebuild(
            &mut views.0.view,
            &mut views.0.children,
            &mut views.0.components,
            context,
        )
    }
}

impl<T1, T2> ElementSeq for (T1, T2)
where
    T1: Element,
    T2: Element,
{
    type Views = (
        ViewPod<T1::View, T1::Components>,
        ViewPod<T2::View, T2::Components>,
    );

    type Widgets = (
        WidgetPod<<T1::View as View>::Widget>,
        WidgetPod<<T2::View as View>::Widget>,
    );

    fn depth() -> usize {
        0.max(T1::View::depth()).max(T2::View::depth())
    }

    fn compile(views: &Self::Views) -> Self::Widgets {
        (T1::compile(&views.0), T2::compile(&views.1))
    }

    fn recompile(views: &Self::Views, widgets: &mut Self::Widgets) -> bool {
        let mut has_changed = false;
        has_changed |= T1::recompile(&views.0, &mut widgets.0.widget, &mut widgets.0.children);
        has_changed |= T2::recompile(&views.1, &mut widgets.1.widget, &mut widgets.1.children);
        has_changed
    }

    fn invalidate(views: &Self::Views, context: &mut Context) {
        T1::invalidate(&views.0, context);
        T2::invalidate(&views.1, context);
    }

    fn build(self, context: &mut Context) -> Self::Views {
        (self.0.build(context), self.1.build(context))
    }

    fn rebuild(self, views: &mut Self::Views, context: &mut Context) -> bool {
        let mut has_changed = false;
        has_changed |= self.0.rebuild(
            &mut views.0.view,
            &mut views.0.children,
            &mut views.0.components,
            context,
        );
        has_changed |= self.1.rebuild(
            &mut views.1.view,
            &mut views.1.children,
            &mut views.1.components,
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
    type Views = (
        ViewPod<T1::View, T1::Components>,
        ViewPod<T2::View, T2::Components>,
        ViewPod<T3::View, T3::Components>,
    );

    type Widgets = (
        WidgetPod<<T1::View as View>::Widget>,
        WidgetPod<<T2::View as View>::Widget>,
        WidgetPod<<T3::View as View>::Widget>,
    );

    fn depth() -> usize {
        0.max(T1::View::depth()).max(T2::View::depth()).max(T3::View::depth())
    }

    fn compile(views: &Self::Views) -> Self::Widgets {
        (
            T1::compile(&views.0),
            T2::compile(&views.1),
            T3::compile(&views.2),
        )
    }

    fn recompile(views: &Self::Views, widgets: &mut Self::Widgets) -> bool {
        let mut has_changed = false;
        has_changed |= T1::recompile(&views.0, &mut widgets.0.widget, &mut widgets.0.children);
        has_changed |= T2::recompile(&views.1, &mut widgets.1.widget, &mut widgets.1.children);
        has_changed |= T3::recompile(&views.2, &mut widgets.2.widget, &mut widgets.2.children);
        has_changed
    }

    fn invalidate(views: &Self::Views, context: &mut Context) {
        T1::invalidate(&views.0, context);
        T2::invalidate(&views.1, context);
        T3::invalidate(&views.2, context);
    }

    fn build(self, context: &mut Context) -> Self::Views {
        (self.0.build(context), self.1.build(context), self.2.build(context))
    }

    fn rebuild(self, views: &mut Self::Views, context: &mut Context) -> bool {
        let mut has_changed = false;
        has_changed |= self.0.rebuild(
            &mut views.0.view,
            &mut views.0.children,
            &mut views.0.components,
            context,
        );
        has_changed |= self.1.rebuild(
            &mut views.1.view,
            &mut views.1.children,
            &mut views.1.components,
            context,
        );
        has_changed |= self.2.rebuild(
            &mut views.2.view,
            &mut views.2.children,
            &mut views.2.components,
            context,
        );
        has_changed
    }
}

impl<T> ElementSeq for Option<T>
where
    T: Element,
{
    type Views = Option<ViewPod<T::View, T::Components>>;

    type Widgets = Option<WidgetPod<<T::View as View>::Widget>>;

    fn depth() -> usize {
        T::View::depth()
    }

    fn compile(views: &Self::Views) -> Self::Widgets {
        views.as_ref().map(|view_pod| T::compile(view_pod))
    }

    fn recompile(views: &Self::Views, widgets: &mut Self::Widgets) -> bool {
        match (views, widgets.as_mut()) {
            (Some(view_pod), Some(widget_pod)) => {
                T::recompile(view_pod, &mut widget_pod.widget, &mut widget_pod.children)
            }
            (Some(view_pod), None) => {
                *widgets = Some(T::compile(view_pod));
                true
            }
            (None, Some(_)) => {
                *widgets = None;
                true
            }
            (None, None) => false,
        }
    }

    fn invalidate(views: &Self::Views, context: &mut Context) {
        if let Some(view_pod) = views {
            T::invalidate(view_pod, context);
        }
    }

    fn build(self, context: &mut Context) -> Self::Views {
        self.map(|el| el.build(context))
    }

    fn rebuild(self, views: &mut Self::Views, context: &mut Context) -> bool {
        match (self, views.as_mut()) {
            (Some(el), Some(view_pod)) => el.rebuild(
                &mut view_pod.view,
                &mut view_pod.children,
                &mut view_pod.components,
                context,
            ),
            (Some(el), None) => {
                *views = Some(el.build(context));
                true
            }
            (None, Some(view_pod)) => {
                T::invalidate(view_pod, context);
                *views = None;
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
    type Views = Vec<ViewPod<T::View, T::Components>>;

    type Widgets = Vec<WidgetPod<<T::View as View>::Widget>>;

    fn depth() -> usize {
        T::View::depth()
    }

    fn compile(views: &Self::Views) -> Self::Widgets {
        views.into_iter().map(T::compile).collect()
    }

    fn recompile(views: &Self::Views, widgets: &mut Self::Widgets) -> bool {
        if views.len() < widgets.len() {
            widgets.drain(widgets.len() - views.len() - 1..);
        } else {
            widgets.reserve_exact(views.len());
        }

        let reuse_len = views.len().min(widgets.len());
        let mut has_changed = false;

        for (i, view_pod) in views.into_iter().enumerate() {
            if i < reuse_len {
                let widget_pod = &mut widgets[i];
                if T::recompile(view_pod, &mut widget_pod.widget, &mut widget_pod.children) {
                    has_changed = true;
                }
            } else {
                let widget_pod = T::compile(view_pod);
                widgets.push(widget_pod);
                has_changed = true;
            }
        }

        has_changed
    }

    fn invalidate(views: &Self::Views, context: &mut Context) {
        for view_pod in views {
            T::invalidate(view_pod, context);
        }
    }

    fn build(self, context: &mut Context) -> Self::Views {
        self.into_iter().map(|el| el.build(context)).collect()
    }

    fn rebuild(self, views: &mut Self::Views, context: &mut Context) -> bool {
        if self.len() < views.len() {
            for view_pod in views.drain(views.len() - self.len() - 1..) {
                T::invalidate(&view_pod, context);
            }
        } else {
            views.reserve_exact(self.len());
        }

        let reuse_len = self.len().min(views.len());
        let mut has_changed = false;

        for (i, el) in self.into_iter().enumerate() {
            if i < reuse_len {
                let view_pod = &mut views[i];
                if el.rebuild(
                    &mut view_pod.view,
                    &mut view_pod.children,
                    &mut view_pod.components,
                    context,
                ) {
                    has_changed = true;
                }
            } else {
                let view_pod = el.build(context);
                views.push(view_pod);
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
    type Views = Either<ViewPod<L::View, L::Components>, ViewPod<R::View, R::Components>>;

    type Widgets =
        Either<WidgetPod<<L::View as View>::Widget>, WidgetPod<<R::View as View>::Widget>>;

    fn depth() -> usize {
        L::View::depth().max(R::View::depth())
    }

    fn compile(views: &Self::Views) -> Self::Widgets {
        match views {
            Either::Left(view_pod) => Either::Left(L::compile(view_pod)),
            Either::Right(view_pod) => Either::Right(R::compile(view_pod)),
        }
    }

    fn recompile(views: &Self::Views, widgets: &mut Self::Widgets) -> bool {
        match (views, widgets.as_mut()) {
            (Either::Left(view_pod), Either::Left(widget_pod)) => {
                L::recompile(view_pod, &mut widget_pod.widget, &mut widget_pod.children)
            }
            (Either::Right(view_pod), Either::Right(widget_pod)) => {
                R::recompile(view_pod, &mut widget_pod.widget, &mut widget_pod.children)
            }
            (Either::Left(view_pod), Either::Right(_)) => {
                *widgets = Either::Left(L::compile(view_pod));
                true
            }
            (Either::Right(view_pod), Either::Left(_)) => {
                *widgets = Either::Right(R::compile(view_pod));
                true
            }
        }
    }

    fn invalidate(views: &Self::Views, context: &mut Context) {
        match views {
            Either::Left(el) => L::invalidate(el, context),
            Either::Right(el) => R::invalidate(el, context),
        }
    }

    fn build(self, context: &mut Context) -> Self::Views {
        match self {
            Either::Left(el) => Either::Left(el.build(context)),
            Either::Right(el) => Either::Right(el.build(context)),
        }
    }

    fn rebuild(self, views: &mut Self::Views, context: &mut Context) -> bool {
        match (self, views.as_mut()) {
            (Either::Left(el), Either::Left(view_pod)) => el.rebuild(
                &mut view_pod.view,
                &mut view_pod.children,
                &mut view_pod.components,
                context,
            ),
            (Either::Right(el), Either::Right(view_pod)) => el.rebuild(
                &mut view_pod.view,
                &mut view_pod.children,
                &mut view_pod.components,
                context,
            ),
            (Either::Left(el), Either::Right(view_pod)) => {
                R::invalidate(view_pod, context);
                *views = Either::Left(el.build(context));
                true
            }
            (Either::Right(el), Either::Left(view_pod)) => {
                L::invalidate(view_pod, context);
                *views = Either::Right(el.build(context));
                true
            }
        }
    }
}
