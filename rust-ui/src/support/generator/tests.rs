use super::*;

#[test]
fn test_resume() {
    let mut gen = Generator::new(|co| async move {
        let x = co.suspend("foo").await;
        let y = co.suspend("foobar").await;
        let z = co.suspend("foobarbaz").await;
        [x, y, z]
    });

    assert_eq!(gen.is_complete(), false);

    let x = gen.start().yielded().unwrap();
    let y = gen.resume(x.len()).yielded().unwrap();
    let z = gen.resume(y.len()).yielded().unwrap();
    let result = gen.resume(z.len()).complete().unwrap();

    assert_eq!(x, "foo");
    assert_eq!(y, "foobar");
    assert_eq!(z, "foobarbaz");
    assert_eq!(result, [3, 6, 9]);
    assert_eq!(gen.is_complete(), true);
}

#[test]
fn test_iter() {
    let odd_numbers_less_than_ten = Generator::new(|co| async move {
        let mut n = 1;
        while n < 10 {
            co.suspend(n).await;
            n += 2;
        }
        ()
    });

    assert_eq!(
        odd_numbers_less_than_ten.into_iter().collect::<Vec<_>>(),
        [1, 3, 5, 7, 9]
    );
}
