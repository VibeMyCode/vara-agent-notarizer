//! Notarizer service — on-chain notarization and attestation for AI agents.
//!
//! - Notarize: store a hash + metadata with block timestamp. Paid, returns receipt_id.
//! - Verify: check a receipt exists with the given hash.
//! - Attest: one agent attests something about another. Paid.
//! - Owner-gated fee management.

use crate::types::*;
use sails_rs::cell::RefCell;
use sails_rs::collections::BTreeMap;
use sails_rs::gstd::{exec, msg};
use sails_rs::prelude::*;

// ---------------------------------------------------------------------------
// State
// ---------------------------------------------------------------------------

#[derive(Default)]
pub struct NotarizerState {
    pub config: Config,
    pub next_receipt_id: ReceiptId,
    pub next_attestation_id: AttestationId,
    pub receipts: BTreeMap<ReceiptId, Receipt>,
    pub author_receipts: BTreeMap<ActorId, Vec<ReceiptId>>,
    pub attestations: BTreeMap<ActorId, Vec<Attestation>>,
}

// ---------------------------------------------------------------------------
// Events
// ---------------------------------------------------------------------------

#[sails_rs::event]
#[derive(Encode, Decode, TypeInfo, Clone, Debug, PartialEq, Eq)]
#[codec(crate = sails_rs::scale_codec)]
#[scale_info(crate = sails_rs::scale_info)]
pub enum NotarizerEvent {
    ReceiptCreated {
        id: ReceiptId,
        author: ActorId,
        hash: [u8; 32],
        metadata: String,
        block: u32,
        ts: u64,
        value_paid: u128,
    },
    AttestationCreated {
        id: AttestationId,
        subject: ActorId,
        attestor: ActorId,
        claim: String,
        ts: u64,
        value_paid: u128,
    },
    FeeUpdated {
        field: u8, // 0 = notarize, 1 = attest
        old_value: u128,
        new_value: u128,
    },
}

// ---------------------------------------------------------------------------
// Service
// ---------------------------------------------------------------------------

pub struct NotarizerService<'a> {
    state: &'a RefCell<NotarizerState>,
}

impl<'a> NotarizerService<'a> {
    pub fn new(state: &'a RefCell<NotarizerState>) -> Self {
        Self { state }
    }

    fn ensure_owner(&self) -> Result<(), NotarizerError> {
        let caller = msg::source();
        let owner = self.state.borrow().config.owner;
        if caller != owner {
            return Err(NotarizerError::NotOwner);
        }
        Ok(())
    }

    fn check_value(&self, min_value: u128) -> Result<u128, NotarizerError> {
        let value = msg::value();
        if value < min_value {
            return Err(NotarizerError::ValueTooLow);
        }
        Ok(value)
    }
}

#[sails_rs::service(events = NotarizerEvent)]
impl<'a> NotarizerService<'a> {
    /// Notarize a hash with metadata. Requires payment of fee_notarize (default 1 TVARA).
    /// Overpayment stays as donation to the program. Returns the receipt_id.
    #[export]
    pub fn notarize(
        &mut self,
        hash: [u8; 32],
        metadata: String,
    ) -> Result<ReceiptId, NotarizerError> {
        // Validate hash (no all-zero hash)
        if hash.iter().all(|b| *b == 0) {
            return Err(NotarizerError::InvalidHash);
        }

        // Validate metadata
        if metadata.is_empty() {
            return Err(NotarizerError::EmptyMetadata);
        }
        let max_meta = self.state.borrow().config.max_metadata_len as usize;
        if metadata.len() > max_meta {
            return Err(NotarizerError::FieldTooLarge);
        }

        // Check payment
        let fee = self.state.borrow().config.fee_notarize;
        let value = self.check_value(fee)?;

        let caller = msg::source();
        let block = exec::block_height();
        let now = exec::block_timestamp();
        let mut state = self.state.borrow_mut();

        // Generate receipt ID
        state.next_receipt_id = state
            .next_receipt_id
            .checked_add(1)
            .expect("receipt_id overflow");
        let id = state.next_receipt_id;

        let receipt = Receipt {
            id,
            author: caller,
            hash,
            metadata: metadata.clone(),
            block,
            ts: now,
        };

        state.receipts.insert(id, receipt);
        state.author_receipts.entry(caller).or_default().push(id);

        drop(state);

        self.emit_event(NotarizerEvent::ReceiptCreated {
            id,
            author: caller,
            hash,
            metadata,
            block,
            ts: now,
            value_paid: value,
        })
        .expect("emit ReceiptCreated failed");

        Ok(id)
    }

    /// Verify that a receipt exists with the given hash.
    #[export]
    pub fn verify(&self, receipt_id: ReceiptId, hash: [u8; 32]) -> bool {
        self.state
            .borrow()
            .receipts
            .get(&receipt_id)
            .map_or(false, |r| r.hash == hash)
    }

    /// Get a single receipt by ID.
    #[export]
    pub fn get_receipt(&self, receipt_id: ReceiptId) -> Option<Receipt> {
        self.state.borrow().receipts.get(&receipt_id).cloned()
    }

    /// List receipts by author, paginated.
    #[export]
    pub fn get_receipts_by_author(
        &self,
        author: ActorId,
        cursor: Option<ReceiptId>,
        limit: u32,
    ) -> ReceiptPage {
        let limit = limit.min(MAX_PAGE_SIZE) as usize;
        let state = self.state.borrow();
        let mut items = Vec::with_capacity(limit);
        let mut next_cursor = None;

        if let Some(ids) = state.author_receipts.get(&author) {
            for receipt_id in ids.iter() {
                if cursor.map_or(false, |c| *receipt_id <= c) {
                    continue;
                }
                if items.len() == limit {
                    next_cursor = Some(*receipt_id);
                    break;
                }
                if let Some(receipt) = state.receipts.get(receipt_id) {
                    items.push(receipt.clone());
                }
            }
        }

        ReceiptPage { items, next_cursor }
    }

    /// Attest something about another agent. Requires payment of fee_attest (default 0.5 TVARA).
    /// Overpayment stays as donation. Returns the attestation_id.
    #[export]
    pub fn attest(
        &mut self,
        subject: ActorId,
        claim: String,
    ) -> Result<AttestationId, NotarizerError> {
        if subject == ActorId::zero() {
            return Err(NotarizerError::InvalidHash);
        }
        if claim.is_empty() {
            return Err(NotarizerError::EmptyClaim);
        }
        let max_claim = self.state.borrow().config.max_claim_len as usize;
        if claim.len() > max_claim {
            return Err(NotarizerError::FieldTooLarge);
        }

        let fee = self.state.borrow().config.fee_attest;
        let value = self.check_value(fee)?;

        let caller = msg::source();
        let now = exec::block_timestamp();
        let mut state = self.state.borrow_mut();

        state.next_attestation_id = state
            .next_attestation_id
            .checked_add(1)
            .expect("attestation_id overflow");
        let id = state.next_attestation_id;

        let attestation = Attestation {
            id,
            subject,
            attestor: caller,
            claim: claim.clone(),
            ts: now,
        };

        state.attestations.entry(subject).or_default().push(attestation);
        drop(state);

        self.emit_event(NotarizerEvent::AttestationCreated {
            id,
            subject,
            attestor: caller,
            claim,
            ts: now,
            value_paid: value,
        })
        .expect("emit AttestationCreated failed");

        Ok(id)
    }

    /// Get all attestations for a subject, paginated.
    #[export]
    pub fn get_attestations(
        &self,
        subject: ActorId,
        cursor: Option<AttestationId>,
        limit: u32,
    ) -> AttestationPage {
        let limit = limit.min(MAX_PAGE_SIZE) as usize;
        let state = self.state.borrow();
        let mut items = Vec::with_capacity(limit);
        let mut next_cursor = None;

        if let Some(attestations) = state.attestations.get(&subject) {
            for att in attestations.iter() {
                if cursor.map_or(false, |c| att.id <= c) {
                    continue;
                }
                if items.len() == limit {
                    next_cursor = Some(att.id);
                    break;
                }
                items.push(att.clone());
            }
        }

        AttestationPage { items, next_cursor }
    }

    /// Owner-only: set fee for notarize.
    #[export]
    pub fn set_fee_notarize(&mut self, new_fee: u128) -> Result<(), NotarizerError> {
        self.ensure_owner()?;
        if new_fee == 0 {
            return Err(NotarizerError::ValueTooLow);
        }
        let mut state = self.state.borrow_mut();
        let old = state.config.fee_notarize;
        state.config.fee_notarize = new_fee;
        drop(state);

        self.emit_event(NotarizerEvent::FeeUpdated {
            field: 0,
            old_value: old,
            new_value: new_fee,
        })
        .expect("emit FeeUpdated failed");

        Ok(())
    }

    /// Owner-only: set fee for attest.
    #[export]
    pub fn set_fee_attest(&mut self, new_fee: u128) -> Result<(), NotarizerError> {
        self.ensure_owner()?;
        if new_fee == 0 {
            return Err(NotarizerError::ValueTooLow);
        }
        let mut state = self.state.borrow_mut();
        let old = state.config.fee_attest;
        state.config.fee_attest = new_fee;
        drop(state);

        self.emit_event(NotarizerEvent::FeeUpdated {
            field: 1,
            old_value: old,
            new_value: new_fee,
        })
        .expect("emit FeeUpdated failed");

        Ok(())
    }

    /// Get current config (free query).
    #[export]
    pub fn get_config(&self) -> Config {
        self.state.borrow().config.clone()
    }
}
