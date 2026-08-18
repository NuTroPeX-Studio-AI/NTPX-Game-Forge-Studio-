use std::{collections::BTreeSet, error::Error, fmt, path::PathBuf};

use crate::{
    resolve_filesystem_target, ApprovalContext, GovernanceRuntime, GovernedFileTransaction, Request,
    RuntimeError, TransactionError, TransactionReceipt,
};

/// One owned filesystem mutation in a governed multi-file transaction set.
#[derive(Debug)]
pub struct BatchFileWrite<'a> {
    pub request: Request,
    pub target: PathBuf,
    pub contents: Vec<u8>,
    pub transaction_id: String,
    pub approval: Option<ApprovalContext<'a>>,
}

/// Receipt for an all-or-reverted governed transaction set.
#[derive(Debug)]
pub struct BatchTransactionReceipt {
    pub batch_id: String,
    pub receipts: Vec<TransactionReceipt>,
}

/// Errors produced by governed multi-file transaction sets.
#[derive(Debug)]
pub enum BatchTransactionError {
    EmptyBatch,
    InvalidBatchId,
    DuplicateTarget(PathBuf),
    Runtime(RuntimeError),
    Rollback {
        source: Box<RuntimeError>,
        rollback: Box<TransactionError>,
    },
}

impl fmt::Display for BatchTransactionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyBatch => write!(formatter, "governed transaction batch is empty"),
            Self::InvalidBatchId => write!(formatter, "governed transaction batch id is invalid"),
            Self::DuplicateTarget(path) => write!(
                formatter,
                "governed transaction batch contains duplicate canonical target '{}'",
                path.display()
            ),
            Self::Runtime(error) => write!(formatter, "governed batch write failed: {error}"),
            Self::Rollback { source, rollback } => write!(
                formatter,
                "governed batch write failed ({source}) and rollback also failed ({rollback})"
            ),
        }
    }
}

impl Error for BatchTransactionError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Runtime(error) => Some(error),
            Self::Rollback { source, .. } => Some(source),
            Self::EmptyBatch | Self::InvalidBatchId | Self::DuplicateTarget(_) => None,
        }
    }
}

impl From<RuntimeError> for BatchTransactionError {
    fn from(error: RuntimeError) -> Self {
        Self::Runtime(error)
    }
}

/// Multi-file coordinator built exclusively on the verified canonical runtime and reversible
/// single-file transaction primitive.
///
/// Writes are committed in caller order. If any write fails, every earlier successful write is
/// rolled back in reverse order before the error is returned. This provides all-or-reverted
/// filesystem state for ordinary failures. It is not a kernel-level multi-file atomic rename;
/// crash recovery remains a separate, explicit Phase 03 recovery concern.
#[derive(Debug, Clone, Copy, Default)]
pub struct GovernedFileBatch;

impl GovernedFileBatch {
    /// Commit one governed multi-file transaction set.
    ///
    /// Each item may use either direct policy authorization or a request-bound human approval
    /// context. Canonical duplicate targets are rejected before any mutation.
    ///
    /// # Errors
    /// Returns [`BatchTransactionError`] for invalid batch metadata, path resolution failure,
    /// duplicate targets, authorization/transaction failure, or rollback failure.
    pub fn commit(
        runtime: &GovernanceRuntime<'_>,
        batch_id: &str,
        writes: Vec<BatchFileWrite<'_>>,
    ) -> Result<BatchTransactionReceipt, BatchTransactionError> {
        validate_batch_id(batch_id)?;
        if writes.is_empty() {
            return Err(BatchTransactionError::EmptyBatch);
        }
        validate_unique_targets(&writes)?;

        let mut receipts = Vec::with_capacity(writes.len());
        for write in writes {
            let result = match write.approval {
                Some(approval) => runtime.write_file_with_approval(
                    write.request,
                    &write.target,
                    &write.contents,
                    &write.transaction_id,
                    approval,
                ),
                None => runtime.write_file(
                    write.request,
                    &write.target,
                    &write.contents,
                    &write.transaction_id,
                ),
            };

            match result {
                Ok(receipt) => receipts.push(receipt),
                Err(source) => {
                    if let Err(rollback) = rollback_receipts(&receipts) {
                        return Err(BatchTransactionError::Rollback {
                            source: Box::new(source),
                            rollback: Box::new(rollback),
                        });
                    }
                    return Err(BatchTransactionError::Runtime(source));
                }
            }
        }

        Ok(BatchTransactionReceipt {
            batch_id: batch_id.to_owned(),
            receipts,
        })
    }

    /// Roll back an entire committed transaction set in reverse order.
    ///
    /// # Errors
    /// Returns the first [`TransactionError`] encountered while restoring a member snapshot.
    pub fn rollback(receipt: &BatchTransactionReceipt) -> Result<(), TransactionError> {
        rollback_receipts(&receipt.receipts)
    }
}

fn validate_batch_id(value: &str) -> Result<(), BatchTransactionError> {
    let valid = !value.is_empty()
        && value.len() <= 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'));
    if valid {
        Ok(())
    } else {
        Err(BatchTransactionError::InvalidBatchId)
    }
}

fn validate_unique_targets(writes: &[BatchFileWrite<'_>]) -> Result<(), BatchTransactionError> {
    let mut seen = BTreeSet::new();
    for write in writes {
        let resolved = resolve_filesystem_target(&write.target)?;
        let canonical = resolved.path().to_owned();
        if !seen.insert(canonical.clone()) {
            return Err(BatchTransactionError::DuplicateTarget(canonical));
        }
    }
    Ok(())
}

fn rollback_receipts(receipts: &[TransactionReceipt]) -> Result<(), TransactionError> {
    for receipt in receipts.iter().rev() {
        GovernedFileTransaction::rollback(receipt)?;
    }
    Ok(())
}
