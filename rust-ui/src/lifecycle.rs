#[derive(Debug)]
pub enum Lifecycle<Widget, Children> {
    OnMount(Children),
    OnUpdate(Children, Widget, Children),
    OnUnmount(Children),
}

impl<Widget, Children> Lifecycle<Widget, Children> {
    pub fn map<F, NewWidget>(self, f: F) -> Lifecycle<NewWidget, Children>
    where
        F: Fn(Widget) -> NewWidget,
    {
        match self {
            Lifecycle::OnMount(children) => Lifecycle::OnMount(children),
            Lifecycle::OnUpdate(children, new_widget, new_children) => {
                Lifecycle::OnUpdate(children, f(new_widget), new_children)
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
