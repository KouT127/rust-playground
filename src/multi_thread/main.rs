use crate::FutureResult::{Done, Message};
use std::fmt::Error;
use std::time::Instant;
use tokio::prelude::*;
use tokio::stream::StreamExt;
use tokio::sync::mpsc;
use tokio::time::{delay_for, Duration};

enum FutureResult<T> {
    Message(T),
    Done,
}

#[tokio::main]
async fn main() {
    let start = Instant::now();

    let (mut tx, mut rx) = mpsc::channel(100);

    let mut cloned_tx = tx.clone();
    tokio::spawn(async move {
        for i in 0..10 {
            delay_for(Duration::from_millis(800)).await;
            cloned_tx.send(Message(i)).await;
        }
    });
    let mut cloned_tx = tx.clone();
    tokio::spawn(async move {
        for i in 0..10 {
            delay_for(Duration::from_millis(800)).await;
            cloned_tx.send(Message(i)).await;
        }
    });
    let mut cloned_tx = tx.clone();
    tokio::spawn(async move {
        for i in 0..10 {
            delay_for(Duration::from_millis(800)).await;
            cloned_tx.send(Message(i)).await;
        }
        cloned_tx.send(Done).await;
    });

    while let Some(msg) = rx.recv().await {
        match msg {
            Message(message) => println!("{}", message),
            Done => break,
        }
    }
    let end = start.elapsed();
    println!(
        "{}.{:03}秒経過しました。",
        end.as_secs(),
        end.subsec_nanos() / 1_000_000
    );
}
