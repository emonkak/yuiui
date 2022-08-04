use crate::component::Component;
use crate::element::Element;
use crate::view::View;
use crate::world::{Id, World};

pub trait ElementSeq: 'static {
    fn len(&self) -> usize;

    fn placement(self, origin: Id, world: &mut World);

    fn reconcile(self, target: Id, world: &mut World) -> Option<Id>;
}

impl ElementSeq for () {
    fn len(&self) -> usize {
        0
    }

    fn placement(self, _origin: Id, _world: &mut World) {}

    fn reconcile(self, _target: Id, _world: &mut World) -> Option<Id> {
        None
    }
}

impl<V1, C1> ElementSeq for (Element<V1, C1>,)
where
    V1: View,
    C1: Component,
{
    fn len(&self) -> usize {
        1
    }

    fn placement(self, origin: Id, world: &mut World) {
        world.append(origin, self.0);
    }

    fn reconcile(self, target: Id, world: &mut World) -> Option<Id> {
        world.update(target, self.0)
    }
}

impl<V1, V2, C1, C2> ElementSeq for (Element<V1, C1>, Element<V2, C2>)
where
    V1: View,
    V2: View,
    C1: Component,
    C2: Component,
{
    fn len(&self) -> usize {
        2
    }

    fn placement(self, origin: Id, world: &mut World) {
        world.append(origin, self.0);
        world.append(origin, self.1);
    }

    fn reconcile(self, target: Id, world: &mut World) -> Option<Id> {
        let target = world.update(target, self.0).unwrap();
        world.update(target, self.1)
    }
}

impl<V1, V2, V3, C1, C2, C3> ElementSeq for (Element<V1, C1>, Element<V2, C2>, Element<V3, C3>)
where
    V1: View,
    V2: View,
    V3: View,
    C1: Component,
    C2: Component,
    C3: Component,
{
    fn len(&self) -> usize {
        3
    }

    fn placement(self, origin: Id, world: &mut World) {
        world.append(origin, self.0);
        world.append(origin, self.1);
        world.append(origin, self.2);
    }

    fn reconcile(self, target: Id, world: &mut World) -> Option<Id> {
        let target = world.update(target, self.0).unwrap();
        let target = world.update(target, self.1).unwrap();
        world.update(target, self.2)
    }
}

impl<V: View, C: Component> ElementSeq for Option<Element<V, C>> {
    fn len(&self) -> usize {
        if self.is_some() {
            1
        } else {
            0
        }
    }

    fn placement(self, origin: Id, world: &mut World) {
        if let Some(el) = self {
            world.append(origin, el);
        }
    }

    fn reconcile(self, target: Id, world: &mut World) -> Option<Id> {
        if let Some(el) = self {
            world.update(target, el)
        } else {
            world.remove(target)
        }
    }
}

impl<V: View, C: Component> ElementSeq for Vec<Element<V, C>> {
    fn len(&self) -> usize {
        self.len()
    }

    fn placement(self, origin: Id, world: &mut World) {
        for el in self {
            world.append(origin, el);
        }
    }

    fn reconcile(self, _target: Id, _world: &mut World) -> Option<Id> {
        None
    }
}

#[derive(Debug, Clone)]
pub enum Either<L, R> {
    Left(L),
    Right(R),
}

impl<V1, V2, C1, C2> ElementSeq for Either<Element<V1, C1>, Element<V2, C2>>
where
    V1: View,
    V2: View,
    C1: Component,
    C2: Component,
{
    fn len(&self) -> usize {
        1
    }

    fn placement(self, origin: Id, world: &mut World) {
        match self {
            Either::Left(el) => world.append(origin, el),
            Either::Right(el) => world.append(origin, el),
        }
    }

    fn reconcile(self, _target: Id, _world: &mut World) -> Option<Id> {
        None
    }
}
