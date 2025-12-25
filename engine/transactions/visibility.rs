use crate::transactions::manager::TransactionId;
use crate::transactions::snapshot::Snapshot;

pub struct VisibilityChecker;

impl VisibilityChecker {
    pub fn new() -> Self {
        Self
    }

    pub fn is_tuple_visible(
        &self,
        snapshot: &Snapshot,
        xmin: TransactionId,
        xmax: Option<TransactionId>,
    ) -> bool {
        snapshot.is_visible(xmin, xmax)
    }

    pub fn can_update(
        &self,
        snapshot: &Snapshot,
        tuple_xmax: Option<TransactionId>,
    ) -> bool {
        if let Some(xmax) = tuple_xmax {
            if xmax.0 != 0 {
                return false;
            }
        }
        true
    }
}
