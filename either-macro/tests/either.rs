use either::Either;
use either_macro::either;

#[test]
fn test_if() {
    let f = |x| either! {
        if x {
            "foo"
        } else {
            let x = 123;
            x
        }
    };
    assert_eq!(f(true), Either::Left("foo"));
    assert_eq!(f(false), Either::Right(123));

    let f = |x| either! {
        if x == 0 {
            "foo"
        } else if x == 1 {
            let x = 123;
            x
        } else {
            true
        }
    };
    assert_eq!(f(0), Either::Left("foo"));
    assert_eq!(f(1), Either::Right(Either::Left(123)));
    assert_eq!(f(2), Either::Right(Either::Right(true)));
}

#[test]
fn test_match() {
    let f = |x| either! {
        match x {
            _ => "foo",
        }
    };
    assert_eq!(f(0), "foo");

    let f = |x| either! {
        match x {
            0 => "foo",
            _ => {
                let x = 123;
                x
            }
        }
    };
    assert_eq!(f(0), Either::Left("foo"));
    assert_eq!(f(1), Either::Right(123));

    let f = |x| either! {
        match x {
            0 => "foo",
            1 => {
                let x = 123;
                x
            },
            _ => 456.0,
        }
    };
    assert_eq!(f(0), Either::Left("foo"));
    assert_eq!(f(1), Either::Right(Either::Left(123)));
    assert_eq!(f(2), Either::Right(Either::Right(456.0)));

    let f = |x| either! {
        match x {
            0 => "foo",
            1 => {
                let x = 123;
                x
            },
            _ if x == 2 => 456.0,
            _ => true,
        }
    };
    assert_eq!(f(0), Either::Left("foo"));
    assert_eq!(f(1), Either::Right(Either::Left(123)));
    assert_eq!(f(2), Either::Right(Either::Right(Either::Left(456.0))));
    assert_eq!(f(3), Either::Right(Either::Right(Either::Right(true))));
}
