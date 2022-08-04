use crate::view::View;
use crate::component::Component;
use crate::element::Element;
use crate::world::{Id, World};

pub trait Children: 'static {
    fn attach(self, origin: Id, world: &mut World);

    fn reconcile(self, origin: Id, world: &mut World);
}

impl Children for () {
    fn attach(self, _origin: Id, _world: &mut World) {
    }

    fn reconcile(self, _origin: Id, _world: &mut World) {
    }
}

impl<V1, C1> Children for (Element<V1, C1>,)
where
    V1: View,
    C1: Component,
{
    fn attach(self, origin: Id, world: &mut World) {
        world.attach(origin, self.0);
    }

    fn reconcile(self, origin: Id, world: &mut World) {
        let mut cursor = world.tree.cursor_mut(origin);
        let target = cursor.node().first_child().unwrap();
        world.update(target, self.0);
    }
}

impl<V1, V2, C1, C2> Children for (Element<V1, C1>, Element<V2, C2>)
where
    V1: View,
    V2: View,
    C1: Component,
    C2: Component,
{
    fn attach(self, origin: Id, world: &mut World) {
        world.attach(origin, self.0);
        world.attach(origin, self.1);
    }

    fn reconcile(self, _origin: Id, _world: &mut World) {
    }
}

impl<V1, V2, V3, C1, C2, C3> Children for (Element<V1, C1>, Element<V2, C2>, Element<V3, C3>)
where
    V1: View,
    V2: View,
    V3: View,
    C1: Component,
    C2: Component,
    C3: Component,
{
    fn attach(self, origin: Id, world: &mut World) {
        world.attach(origin, self.0);
        world.attach(origin, self.1);
        world.attach(origin, self.2);
    }

    fn reconcile(self, _origin: Id, _world: &mut World) {
    }
}

impl<V: View, C: Component> Children for Option<Element<V, C>> {
    fn attach(self, origin: Id, world: &mut World) {
        if let Some(el) = self {
            world.attach(origin, el);
        }
    }

    fn reconcile(self, _origin: Id, _world: &mut World) {
    }
}

impl<V: View, C: Component> Children for Vec<Element<V, C>> {
    fn attach(self, origin: Id, world: &mut World) {
        for el in self {
            world.attach(origin, el);
        }
    }

    fn reconcile(self, _origin: Id, _world: &mut World) {
    }
}

#[derive(Debug, Clone)]
pub enum Either<L, R> {
    Left(L),
    Right(R),
}

impl<V1, V2, C1, C2> Children for Either<Element<V1, C1>, Element<V2, C2>>
where
    V1: View,
    V2: View,
    C1: Component,
    C2: Component,
{
    fn attach(self, origin: Id, world: &mut World) {
        match self {
            Either::Left(el) => world.attach(origin, el),
            Either::Right(el) => world.attach(origin, el),
        }
    }

    fn reconcile(self, _origin: Id, _world: &mut World) {
    }
}
