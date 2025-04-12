use anyhow::Result;
use std::thread;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time;

#[tokio::main]
async fn main() -> Result<()> {
    let (tx, rx) = mpsc::channel(32);
    let handle = worker(rx);
    tokio::spawn(async move {
        let mut i = 0;
        loop {
            i += 1;
            tx.send(format!("Task: {i:?}")).await?;
            time::sleep(Duration::from_millis(100)).await;
            if i == 100 {
                break;
            }
        }
        #[allow(unreachable_code)]
        Ok::<(), anyhow::Error>(())
    });
    handle.join().unwrap();
    Ok(())
}

fn worker(mut rx: mpsc::Receiver<String>) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        while let Some(s) = rx.blocking_recv() {
            println!("{}", s);
            let r = expensive_blocking_task(s);
            println!("{}", r);
        }
    })
}

fn expensive_blocking_task(name: String) -> String {
    thread::sleep(Duration::from_millis(800));
    blake3::hash(name.as_bytes()).to_string()
}
