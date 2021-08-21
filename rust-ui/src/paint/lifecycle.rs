#[derive(Debug)]
pub enum Lifecycle<Widget, Children> {
    DidMount(Children),
    DidUpdate(Children, Widget, Children),
    DidUnmount(Children),
}

impl<Widget, Children> Lifecycle<Widget, Children> {
    pub fn map<F, NewWidget>(self, f: F) -> Lifecycle<NewWidget, Children>
    where
        F: Fn(Widget) -> NewWidget,
    {
        match self {
            Lifecycle::DidMount(children) => Lifecycle::DidMount(children),
            Lifecycle::DidUpdate(children, new_widget, new_children) => {
                Lifecycle::DidUpdate(children, f(new_widget), new_children)
            }
            Lifecycle::DidUnmount(children) => Lifecycle::DidUnmount(children),
        }
    }

    pub fn without_params(&self) -> Lifecycle<(), ()> {
        match self {
            Lifecycle::DidMount(_) => Lifecycle::DidMount(()),
            Lifecycle::DidUpdate(_, _, _) => Lifecycle::DidUpdate((), (), ()),
            Lifecycle::DidUnmount(_) => Lifecycle::DidUnmount(()),
        }
    }
}
