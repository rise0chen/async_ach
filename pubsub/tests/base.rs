use async_ach_pubsub::Publisher;
use core::time::Duration;
use futures_executor::ThreadPool;
use futures_util::StreamExt;
use std::thread;

#[test]
fn test() {
    static PUB: Publisher<usize, 2, 2, 2> = Publisher::new(false);
    let sub1 = PUB.subscribe().unwrap();
    let sub2 = PUB.subscribe().unwrap();
    let executor = ThreadPool::new().unwrap();

    assert_eq!(PUB.send(1), 2);
    assert_eq!(PUB.send(2), 2);
    executor.spawn_ok(async move {
        let data1 = sub1.recv().next().await;
        assert_eq!(data1, Some(1));
        let data2 = sub1.recv().next().await;
        assert_eq!(data2, Some(2));
    });
    executor.spawn_ok(async move {
        let data1 = sub2.recv().next().await;
        assert_eq!(data1, Some(1));
        let data2 = sub2.recv().next().await;
        assert_eq!(data2, Some(2));
    });
    thread::sleep(Duration::from_secs(1));
}
