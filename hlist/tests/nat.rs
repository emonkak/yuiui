use hlist::nat::*;

#[test]
fn test_nat() {
    assert_eq!(N0::N, 0);
    assert_eq!(N1::N, 1);
    assert_eq!(N2::N, 2);
    assert_eq!(N3::N, 3);
    assert_eq!(N4::N, 4);
    assert_eq!(N5::N, 5);
    assert_eq!(N6::N, 6);
    assert_eq!(N7::N, 7);
    assert_eq!(N8::N, 8);
    assert_eq!(N9::N, 9);
    assert_eq!(N10::N, 10);
}

#[test]
fn test_add() {
    assert_eq!(<N0 as Add<N0>>::Output::N, 0);
    assert_eq!(<N0 as Add<N1>>::Output::N, 1);
    assert_eq!(<N0 as Add<N2>>::Output::N, 2);
    assert_eq!(<N1 as Add<N0>>::Output::N, 1);
    assert_eq!(<N1 as Add<N1>>::Output::N, 2);
    assert_eq!(<N1 as Add<N2>>::Output::N, 3);
    assert_eq!(<N2 as Add<N0>>::Output::N, 2);
    assert_eq!(<N2 as Add<N1>>::Output::N, 3);
    assert_eq!(<N2 as Add<N2>>::Output::N, 4);
}

#[test]
fn test_sub() {
    assert_eq!(<N0 as Sub<N0>>::Output::N, 0);
    assert_eq!(<N1 as Sub<N0>>::Output::N, 1);
    assert_eq!(<N1 as Sub<N1>>::Output::N, 0);
    assert_eq!(<N2 as Sub<N0>>::Output::N, 2);
    assert_eq!(<N2 as Sub<N1>>::Output::N, 1);
    assert_eq!(<N2 as Sub<N2>>::Output::N, 0);
}
