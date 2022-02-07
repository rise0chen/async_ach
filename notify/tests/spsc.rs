use async_ach_notify::Notify;
use core::time::Duration;
use futures_executor::ThreadPool;
use std::process;
use std::thread;

const TEST_TIMES: usize = 10000;

#[test]
fn test() {
    static NOTIFY_P: Notify = Notify::new();
    static NOTIFY_C: Notify = Notify::new();
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
        assert!(!NOTIFY_C.had_notified());
    });
    executor.spawn_ok(async {
        // Cunsumer_1
        for _ in 0..TEST_TIMES {
            NOTIFY_P.listen().await;
            NOTIFY_C.notify_one();
        }
        println!("Cunsumer_1 finished");
        thread::yield_now();
        assert!(!NOTIFY_P.had_notified());
        process::exit(0);
    });
    thread::sleep(Duration::from_secs(3));
    unreachable!()
}
