use async_ach_pubsub::Publisher;
use core::sync::atomic::{AtomicUsize, Ordering::SeqCst};
use core::time::Duration;
use futures_executor::ThreadPool;
use futures_util::StreamExt;
use std::thread;

#[test]
fn test() {
    static STATE: AtomicUsize = AtomicUsize::new(0);
    static PUB: Publisher<usize, 2, 2> = Publisher::new(false);
    let sub1 = PUB.subscribe().unwrap();
    let sub2 = PUB.subscribe().unwrap();
    let executor = ThreadPool::new().unwrap();

    executor.spawn_ok(async move {
        let data = sub1.recv().next().await;
        assert_eq!(data, Some(1));
        println!("1-1");
        let data = sub1.recv().next().await;
        assert_eq!(data, Some(2));
        println!("1-2");
        let data = sub1.recv().next().await;
        assert_eq!(data, Some(3));
        println!("1-3");
        let data = sub1.recv().next().await;
        assert_eq!(data, Some(4));
        println!("1-4");
        STATE.fetch_add(1, SeqCst);
    });
    executor.spawn_ok(async move {
        let data = sub2.recv().next().await;
        assert_eq!(data, Some(1));
        println!("2-1");
        let data = sub2.recv().next().await;
        assert_eq!(data, Some(2));
        println!("2-2");
        let data = sub2.recv().next().await;
        assert_eq!(data, Some(3));
        println!("2-3");
        let data = sub2.recv().next().await;
        assert_eq!(data, Some(4));
        println!("2-4");
        STATE.fetch_add(1, SeqCst);
    });
    assert_eq!(PUB.send(1), 2);
    thread::sleep(Duration::from_millis(100));
    assert_eq!(PUB.send(2), 2);
    thread::sleep(Duration::from_millis(100));
    assert_eq!(PUB.send(3), 2);
    thread::sleep(Duration::from_millis(100));
    assert_eq!(PUB.send(4), 2);
    thread::sleep(Duration::from_secs(1));
    assert_eq!(STATE.load(SeqCst), 2);
}
