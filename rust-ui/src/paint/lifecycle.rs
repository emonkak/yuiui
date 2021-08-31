#[derive(Debug)]
pub enum Lifecycle<Widget> {
    DidMount(),
    DidUpdate(Widget),
    DidUnmount(),
}

impl<Widget> Lifecycle<Widget> {
    pub fn map<F, NewWidget>(self, f: F) -> Lifecycle<NewWidget>
    where
        F: Fn(Widget) -> NewWidget,
    {
        match self {
            Lifecycle::DidMount() => Lifecycle::DidMount(),
            Lifecycle::DidUpdate(new_widget) => {
                Lifecycle::DidUpdate(f(new_widget))
            }
            Lifecycle::DidUnmount() => Lifecycle::DidUnmount(),
        }
    }
}
