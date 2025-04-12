use std::thread;
use std::time::Duration;
use tokio::{fs, time};
fn main() {
    let handle = thread::spawn(|| {
        // 执行一个future
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        rt.spawn(async {
            println!("Future 1!");
            let content = fs::read_to_string("Cargo.toml").await.unwrap();
            println!("{:?}", content.len());
        });

        rt.spawn(async {
            println!("Future 2!");
            let ret = expensive_blocking_task("Future 2");
            println!("{:?}", ret);
        });

        rt.block_on(async {
            time::sleep(Duration::from_millis(1200)).await;
        })
    });
    handle.join().unwrap();
}

fn expensive_blocking_task(name: &str) -> String {
    thread::sleep(Duration::from_millis(800));
    blake3::hash(name.as_bytes()).to_string()
}
