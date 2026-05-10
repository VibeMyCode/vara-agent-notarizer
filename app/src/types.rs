//! Shared DTOs, enums, errors, and consts for the Notarizer service.
//!
//! All types here are IDL-visible. Keep stable after deploy.

use sails_rs::prelude::*;

// ---------------------------------------------------------------------------
// Config
// ---------------------------------------------------------------------------

#[derive(Encode, Decode, TypeInfo, Clone, Debug, PartialEq, Eq)]
#[codec(crate = sails_rs::scale_codec)]
#[scale_info(crate = sails_rs::scale_info)]
pub struct Config {
    pub owner: ActorId,
    pub fee_notarize: u128,
    pub fee_attest: u128,
    pub max_metadata_len: u32,
    pub max_claim_len: u32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            owner: ActorId::zero(),
            fee_notarize: 1_000_000_000_000,   // 1 TVARA
            fee_attest: 500_000_000_000,        // 0.5 TVARA
            max_metadata_len: 256,
            max_claim_len: 512,
        }
    }
}

// ---------------------------------------------------------------------------
// Receipt
// ---------------------------------------------------------------------------

pub type ReceiptId = u64;
pub type AttestationId = u64;

#[derive(Encode, Decode, TypeInfo, Clone, Debug, PartialEq, Eq)]
#[codec(crate = sails_rs::scale_codec)]
#[scale_info(crate = sails_rs::scale_info)]
pub struct Receipt {
    pub id: ReceiptId,
    pub author: ActorId,
    pub hash: [u8; 32],
    pub metadata: String,
    pub block: u32,
    pub ts: u64,
}

#[derive(Encode, Decode, TypeInfo, Clone, Debug, PartialEq, Eq)]
#[codec(crate = sails_rs::scale_codec)]
#[scale_info(crate = sails_rs::scale_info)]
pub struct ReceiptPage {
    pub items: Vec<Receipt>,
    pub next_cursor: Option<ReceiptId>,
}

// ---------------------------------------------------------------------------
// Attestation
// ---------------------------------------------------------------------------

#[derive(Encode, Decode, TypeInfo, Clone, Debug, PartialEq, Eq)]
#[codec(crate = sails_rs::scale_codec)]
#[scale_info(crate = sails_rs::scale_info)]
pub struct Attestation {
    pub id: AttestationId,
    pub subject: ActorId,
    pub attestor: ActorId,
    pub claim: String,
    pub ts: u64,
}

#[derive(Encode, Decode, TypeInfo, Clone, Debug, PartialEq, Eq)]
#[codec(crate = sails_rs::scale_codec)]
#[scale_info(crate = sails_rs::scale_info)]
pub struct AttestationPage {
    pub items: Vec<Attestation>,
    pub next_cursor: Option<AttestationId>,
}

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[derive(Encode, Decode, TypeInfo, Clone, Copy, Debug, PartialEq, Eq)]
#[codec(crate = sails_rs::scale_codec)]
#[scale_info(crate = sails_rs::scale_info)]
pub enum NotarizerError {
    NotOwner,
    ValueTooLow,
    ReceiptNotFound,
    AttestationNotFound,
    FieldTooLarge,
    InvalidHash,
    EmptyClaim,
    EmptyMetadata,
    InsufficientValue,
}

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

pub const HASH_SIZE: usize = 32;
pub const MAX_PAGE_SIZE: u32 = 50;
