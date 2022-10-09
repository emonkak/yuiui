use std::any::TypeId;
use yuiui::Event;
use yuiui_macros::Event;

#[derive(Debug, Eq, PartialEq)]
struct Foo;

#[derive(Debug, Eq, PartialEq)]
struct Bar;

#[derive(Debug, Eq, PartialEq)]
struct Baz;

#[derive(Debug, Eq, Event, PartialEq)]
enum EnumEvent<'event> {
    Foo(&'event Foo),
    Bar(&'event Bar),
    Baz { baz: &'event Baz },
}

#[test]
fn test_event() {
    let types: Vec<TypeId> = EnumEvent::types().into_iter().collect();
    assert_eq!(
        types,
        vec![
            TypeId::of::<Foo>(),
            TypeId::of::<Bar>(),
            TypeId::of::<Baz>(),
        ]
    );

    assert_eq!(EnumEvent::from_any(&Foo), Some(EnumEvent::Foo(&Foo)));
    assert_eq!(EnumEvent::from_any(&Bar), Some(EnumEvent::Bar(&Bar)));
    assert_eq!(
        EnumEvent::from_any(&Baz),
        Some(EnumEvent::Baz { baz: &Baz })
    );
    assert_eq!(EnumEvent::from_any(&()), None);
}