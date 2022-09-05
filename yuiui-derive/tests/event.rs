use std::any::TypeId;
use yuiui::Event;
use yuiui_derive::Event;

#[derive(Debug, Eq, PartialEq)]
struct Foo;

#[derive(Debug, Eq, PartialEq)]
struct Bar;

#[derive(Debug, Eq, PartialEq)]
struct Baz;

#[derive(Debug, Eq, Event, PartialEq)]
struct StructEvent<'event>(&'event Foo);

#[derive(Debug, Eq, Event, PartialEq)]
struct NamedStructEvent<'event> {
    foo: &'event Foo,
}

#[derive(Debug, Eq, Event, PartialEq)]
enum EnumEvent<'event> {
    Foo(&'event Foo),
    Bar(&'event Bar),
    Baz { baz: &'event Baz },
}

#[test]
fn test_struct_event() {
    let mut types = Vec::new();
    StructEvent::collect_types(&mut types);
    assert_eq!(types, vec![TypeId::of::<Foo>()]);

    assert_eq!(StructEvent::from_any(&Foo), Some(StructEvent(&Foo)));
    assert_eq!(StructEvent::from_any(&Bar), None);
    assert_eq!(StructEvent::from_any(&Baz), None);
}

#[test]
fn test_named_struct_event() {
    let mut types = Vec::new();
    NamedStructEvent::collect_types(&mut types);
    assert_eq!(types, vec![TypeId::of::<Foo>()]);

    assert_eq!(
        NamedStructEvent::from_any(&Foo),
        Some(NamedStructEvent { foo: &Foo })
    );
    assert_eq!(NamedStructEvent::from_any(&Bar), None);
    assert_eq!(NamedStructEvent::from_any(&Baz), None);
}

#[test]
fn test_enum_event() {
    let mut types = Vec::new();
    EnumEvent::collect_types(&mut types);
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
