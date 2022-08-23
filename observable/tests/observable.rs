use futures::executor::block_on;
use futures::future;
use futures::stream::StreamExt as _;
use observable::Observable;

#[test]
fn test_new() {
    let mut observable = Observable::create(|observer| async move {
        observer.on_next(1);
        future::ready(()).await;
        observer.on_next(2);
        future::ready(()).await;
        observer.on_next(3);
    });
    let xs = block_on(async {
        let mut xs = vec![];
        while let Some(x) = observable.next().await {
            xs.push(x)
        }
        xs
    });
    assert_eq!(xs, vec![1, 2, 3]);
}
