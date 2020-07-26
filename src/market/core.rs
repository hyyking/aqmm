use std::sync::Arc;

use super::pool;
use crate::protocol::RequestOrder;

pub type Score = fn(&[pool::Entry]) -> f64;

#[derive(Clone)]
pub struct Core {
    pool: Arc<pool::SecurityPool>,
    score: Score,
}

impl Core {
    pub(crate) fn new(score: Score, pool: Arc<pool::SecurityPool>) -> Self {
        Self { score, pool }
    }

    pub(crate) fn execute(&self, request: RequestOrder) {
        let mut entries = self.pool.access();
        let old_entries: Vec<pool::Entry> = entries.iter().map(Clone::clone).collect();

        for order in request.orders {
            entries[order.security_id as usize].quantity += match order.kind {
                0 => order.amount,
                1 => -order.amount,
                _ => unreachable!("wrong enum"),
            };
        }
        let price = (self.score)(&entries) - (self.score)(&old_entries);
        trace!("price: {}", price);
    }
}
