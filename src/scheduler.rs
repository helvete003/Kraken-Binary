use tokio::time;
use std::time::Duration;
use tokio::sync::mpsc;
use crate::manager;

pub struct Scheduler {
    duration: u64
}

impl Scheduler {
    pub fn new(duration: u64) -> Self {
        Self {
            duration
        }
    }
    pub async fn run(&self, tx: mpsc::Sender<manager::Command>) {
        let mut interval = time::interval(Duration::from_secs(self.duration));
        loop {
            interval.tick().await;
            tx.send(manager::Command::Scheduler("refresh")).await.unwrap();
        }
    }
}