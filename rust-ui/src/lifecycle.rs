#[derive(Debug)]
pub enum Lifecycle<Widget, Children> {
    OnMount(Children),
    OnUpdate(Widget, Children, Children),
    OnUnmount(Children),
}

impl<Widget, Children> Lifecycle<Widget, Children> {
    pub fn map<F, NewWidget>(self, f: F) -> Lifecycle<NewWidget, Children>
    where
        F: Fn(Widget) -> NewWidget,
    {
        match self {
            Lifecycle::OnMount(children) => Lifecycle::OnMount(children),
            Lifecycle::OnUpdate(widget, new_children, old_children) => {
                Lifecycle::OnUpdate(f(widget), new_children, old_children)
            }
            Lifecycle::OnUnmount(children) => Lifecycle::OnUnmount(children),
        }
    }

    pub fn without_params(&self) -> Lifecycle<(), ()> {
        match self {
            Lifecycle::OnMount(_) => Lifecycle::OnMount(()),
            Lifecycle::OnUpdate(_, _, _) => Lifecycle::OnUpdate((), (), ()),
            Lifecycle::OnUnmount(_) => Lifecycle::OnUnmount(()),
        }
    }
}
