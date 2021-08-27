#[derive(Debug)]
pub enum Lifecycle<Widget, Children> {
    DidMount(),
    DidUpdate(Widget, Children),
    DidUnmount(),
}

impl<Widget, Children> Lifecycle<Widget, Children> {
    pub fn map<F, NewWidget>(self, f: F) -> Lifecycle<NewWidget, Children>
    where
        F: Fn(Widget) -> NewWidget,
    {
        match self {
            Lifecycle::DidMount() => Lifecycle::DidMount(),
            Lifecycle::DidUpdate(new_widget, new_children) => {
                Lifecycle::DidUpdate(f(new_widget), new_children)
            }
            Lifecycle::DidUnmount() => Lifecycle::DidUnmount(),
        }
    }
}
