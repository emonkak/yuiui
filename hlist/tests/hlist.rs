use hlist::*;

#[test]
fn test_first() {
    let xs = hlist![123, 456.0, true, "foo"];
    assert_eq!(*get_first(&xs), 123);
}

#[test]
fn test_last() {
    let xs = hlist![123, 456.0, true, "foo"];
    assert_eq!(*get_last(&xs), "foo");
}

fn get_first<T: First<U>, U>(value: &T) -> &U {
    value.first()
}

fn get_last<T: Last<U>, U>(value: &T) -> &U {
    value.last()
}
