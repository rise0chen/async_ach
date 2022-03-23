use async_ach_watch::Watch;
use core::time::Duration;
use futures_executor::ThreadPool;
use futures_util::StreamExt;
use std::sync::Arc;

#[futures_test::test]
async fn test() {
    async_tick::auto_tick(Duration::from_millis(10));
    let executor = ThreadPool::new().unwrap();
    let watch: Arc<Watch<usize, 2>> = Arc::new(Watch::new(0));
    println!("{:p}", watch);
    {
        let watch = watch.clone();
        println!("{:p}", watch);
        executor.spawn_ok(async move {
            let mut sub = watch.subscribe();
            let mut sub = sub.changed_interval(Duration::from_secs(1)..Duration::from_secs(3));
            assert_eq!(sub.next().await, Some(1));
            assert_eq!(sub.next().await, Some(3));
            assert_eq!(sub.next().await, Some(4));
            assert_eq!(sub.next().await, Some(4));
            std::process::exit(0);
        });
    }
    async_tick::sleep(Duration::from_secs(1)).await;
    watch.send(1).await;
    async_tick::sleep(Duration::from_millis(100)).await;
    watch.send(2).await;
    async_tick::sleep(Duration::from_millis(100)).await;
    watch.send(3).await;
    async_tick::sleep(Duration::from_secs(2)).await;
    watch.send(4).await;
    async_tick::sleep(Duration::from_secs(10)).await;
}
