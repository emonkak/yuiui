use crate::view::View;
use crate::component::Component;
use crate::element::Element;
use crate::world::{Id, World};

pub trait Children: 'static {
    fn len(&self) -> usize;

    fn append(self, origin: Id, world: &mut World);

    fn update(self, target: Id, world: &mut World) -> Option<Id>;
}

impl Children for () {
    fn len(&self) -> usize {
        0
    }

    fn append(self, _origin: Id, _world: &mut World) {
    }

    fn update(self, _target: Id, _world: &mut World) -> Option<Id> {
        None
    }
}

impl<V1, C1> Children for (Element<V1, C1>,)
where
    V1: View,
    C1: Component,
{
    fn len(&self) -> usize {
        1
    }

    fn append(self, origin: Id, world: &mut World) {
        world.append(origin, self.0);
    }

    fn update(self, target: Id, world: &mut World) -> Option<Id> {
        world.update(target, 0, self.0)
    }
}

impl<V1, V2, C1, C2> Children for (Element<V1, C1>, Element<V2, C2>)
where
    V1: View,
    V2: View,
    C1: Component,
    C2: Component,
{
    fn len(&self) -> usize {
        2
    }

    fn append(self, origin: Id, world: &mut World) {
        world.append(origin, self.0);
        world.append(origin, self.1);
    }

    fn update(self, target: Id, world: &mut World) -> Option<Id> {
        let target = world.update(target, 0, self.0).unwrap();
        world.update(target, 0, self.1)
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
    fn len(&self) -> usize {
        3
    }

    fn append(self, origin: Id, world: &mut World) {
        world.append(origin, self.0);
        world.append(origin, self.1);
        world.append(origin, self.2);
    }

    fn update(self, target: Id, world: &mut World) -> Option<Id> {
        let target = world.update(target, 0, self.0).unwrap();
        let target = world.update(target, 0, self.1).unwrap();
        world.update(target, 0, self.2)
    }
}

impl<V: View, C: Component> Children for Option<Element<V, C>> {
    fn len(&self) -> usize {
        if self.is_some() { 1 } else { 0 }
    }

    fn append(self, origin: Id, world: &mut World) {
        if let Some(el) = self {
            world.append(origin, el);
        }
    }

    fn update(self, target: Id, world: &mut World) -> Option<Id> {
        if let Some(el) = self {
            world.update(target, 0, el)
        } else {
            world.remove(target, 0)
        }
    }
}

impl<V: View, C: Component> Children for Vec<Element<V, C>> {
    fn len(&self) -> usize {
        self.len()
    }

    fn append(self, origin: Id, world: &mut World) {
        for el in self {
            world.append(origin, el);
        }
    }

    fn update(self, _target: Id, _world: &mut World) -> Option<Id> {
        None
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
    fn len(&self) -> usize {
        1
    }

    fn append(self, origin: Id, world: &mut World) {
        match self {
            Either::Left(el) => world.append(origin, el),
            Either::Right(el) => world.append(origin, el),
        }
    }

    fn update(self, _target: Id, _world: &mut World) -> Option<Id> {
        None
    }
}
