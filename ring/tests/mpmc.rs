use async_ach_ring::Ring;
use core::time::Duration;
use futures_executor::ThreadPool;
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
        // Producer_2
        for i in TEST_TIMES..2 * TEST_TIMES {
            RING.push(i).await;
        }
        println!("Producer_2 finished");
    });
    executor.spawn_ok(async {
        // Cunsumer_1
        for _ in 0..TEST_TIMES {
            RING.pop().await;
        }
        println!("Cunsumer_1 finished");
    });
    executor.spawn_ok(async {
        // Cunsumer_2
        for _ in 0..TEST_TIMES {
            RING.pop().await;
        }
        println!("Cunsumer_2 finished");
    });
    thread::sleep(Duration::from_secs(1));
    assert_eq!(RING.len(), 0)
}
