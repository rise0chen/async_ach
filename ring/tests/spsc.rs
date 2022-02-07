use async_ach_ring::Ring;
use core::time::Duration;
use futures_executor::ThreadPool;
use std::process;
use std::thread;

const TEST_TIMES: usize = 10000;

#[test]
fn test() {
    static RING: Ring<usize, 2> = Ring::new();
    let executor = ThreadPool::new().unwrap();
    executor.spawn_ok(async {
        // Producer_1
        for i in 0..TEST_TIMES {
            RING.push(i).await;
        }
        println!("Producer_1 finished");
    });
    executor.spawn_ok(async {
        // Cunsumer_1
        for i in 0..TEST_TIMES {
            assert_eq!(RING.pop().await, i);
        }
        println!("Cunsumer_1 finished");
        thread::yield_now();
        process::exit(0);
    });
    thread::sleep(Duration::from_secs(3));
    unreachable!()
}
