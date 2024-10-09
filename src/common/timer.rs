use std::future::Future;
use std::mem;
use std::time::Duration;
use tokio::select;
use tokio::sync::{mpsc, oneshot};
use tokio::task::JoinHandle;
use tokio::time::sleep;

pub struct Timer {
    ticker: mpsc::Receiver<i32>,
    cancel_s: Option<oneshot::Sender<i32>>,
}
impl Timer {
    pub fn timer(d: Duration) -> Timer {
        let (tx, rx) = mpsc::channel(500);
        let (tx_o, mut rx_o) = oneshot::channel::<i32>();
        tokio::spawn(async move {
            loop {
                select! {
                    _=sleep(d)=>{
                        if let Err(e) = tx.send(0).await {
                            return;
                        }
                    }
                    _=&mut rx_o=>{
                        return;
                    }
                }
            }
        });
        Timer {
            ticker: rx,
            cancel_s: Some(tx_o),
        }
    }
    pub async fn tick(&mut self) {
        let _ = self.ticker.recv().await;
    }
    pub fn cancel(&mut self) {
        let mut can = None;
        mem::swap(&mut can, &mut self.cancel_s);
        if let Some(e) = can {
            let _ = e.send(0);
        }
    }
}
impl Drop for Timer {
    fn drop(&mut self) {
        self.cancel();
    }
}