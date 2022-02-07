use async_ach_notify::Notify;
use core::sync::atomic::{AtomicUsize, Ordering::SeqCst};
use core::time::Duration;
use futures_executor::ThreadPool;
use std::process;
use std::thread;

const TEST_TIMES: usize = 10000;

#[test]
fn test() {
    static FINISHED: AtomicUsize = AtomicUsize::new(0);
    static NOTIFY_P: Notify<2> = Notify::new();
    static NOTIFY_C: Notify<2> = Notify::new();
    let executor = ThreadPool::new().unwrap();
    executor.spawn_ok(async {
        // Producer_1
        NOTIFY_P.notify_one();
        for _ in 0..TEST_TIMES - 1 {
            NOTIFY_C.listen().await;
            NOTIFY_P.notify_one();
        }
        NOTIFY_C.listen().await;
        println!("Producer_1 finished");
    });
    executor.spawn_ok(async {
        // Producer_2
        NOTIFY_P.notify_one();
        for _ in 0..TEST_TIMES - 1 {
            NOTIFY_C.listen().await;
            NOTIFY_P.notify_one();
        }
        NOTIFY_C.listen().await;
        println!("Producer_2 finished");
    });
    executor.spawn_ok(async {
        // Cunsumer_1
        for _ in 0..TEST_TIMES {
            NOTIFY_P.listen().await;
            NOTIFY_C.notify_one();
        }
        println!("Cunsumer_1 finished");
        if FINISHED.fetch_add(1, SeqCst) == 1 {
            thread::yield_now();
            assert!(!NOTIFY_P.had_notified());
            process::exit(0);
        }
    });
    executor.spawn_ok(async {
        // Cunsumer_2
        for _ in 0..TEST_TIMES {
            NOTIFY_P.listen().await;
            NOTIFY_C.notify_one();
        }
        println!("Cunsumer_2 finished");
        if FINISHED.fetch_add(1, SeqCst) == 1 {
            thread::yield_now();
            assert!(!NOTIFY_P.had_notified());
            process::exit(0);
        }
    });
    thread::sleep(Duration::from_secs(3));
    unreachable!()
}
