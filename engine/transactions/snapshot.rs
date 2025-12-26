use crate::determinism::clock::LogicalTime;
use crate::transactions::manager::TransactionId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    pub xid: TransactionId,
    pub logical_time: LogicalTime,
    pub active_xids: Vec<TransactionId>,
}

impl Snapshot {
    pub fn new(
        xid: TransactionId,
        logical_time: LogicalTime,
        active_xids: Vec<TransactionId>,
    ) -> Self {
        Self {
            xid,
            logical_time,
            active_xids,
        }
    }

    pub fn is_visible(&self, tuple_xmin: TransactionId, tuple_xmax: Option<TransactionId>) -> bool {
        if tuple_xmin.0 > self.xid.0 {
            return false;
        }

        if self.active_xids.contains(&tuple_xmin) && tuple_xmin != self.xid {
            return false;
        }

        if let Some(xmax) = tuple_xmax {
            if xmax.0 == 0 {
                return true;
            }

            if xmax.0 > self.xid.0 {
                return true;
            }

            if self.active_xids.contains(&xmax) && xmax != self.xid {
                return true;
            }

            return false;
        }

        true
    }
}
