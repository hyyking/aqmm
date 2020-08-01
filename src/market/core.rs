use std::sync::Arc;

use super::pool;
use super::router;
use crate::protocol::{RequestOrder, ResponseOrder};

use futures::channel::oneshot::Sender;

pub type ScoreFn = fn(&[pool::Entry]) -> f64;

pub(super) struct Task {
    pub(super) req: RequestOrder,
    pub(super) sender: Sender<ResponseOrder>,
}

pub struct Core {
    pub(super) pool: Arc<pool::SecurityPool>,
    pub(super) score: ScoreFn,
    pub(super) receiver: router::EndPoint<Task>,
}

pub async fn run(mut core: Core) {
    use futures::StreamExt;

    loop {
        match core.receiver.next().await {
            Some(task) => {
                let Task { sender, req } = task;

                if let Err(e) = core.execute(req) {
                    error!("Core error: {:?}", e);
                }
                debug_assert!(sender.send(ResponseOrder { orders: vec![] }).is_ok());
            }
            None => break,
        }
    }
}

impl Core {
    pub(crate) fn execute(&self, request: RequestOrder) -> Result<(), &'static str> {
        let mut entries = self.pool.access();

        let old_entries = (self.score)(&entries);

        for order in request.orders {
            match entries.get_mut(order.security_id as usize) {
                Some(ref mut entry) => {
                    entry.quantity += match order.kind {
                        0 => order.amount,
                        1 => -order.amount,
                        _ => unreachable!("wrong enum"),
                    };
                }
                None => return Err("out of index order"),
            }
        }
        let price = (self.score)(&entries) - old_entries;
        trace!("price: {}", price);
        Ok(())
    }
}
