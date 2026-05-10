#![no_std]

extern crate alloc;

use sails_rs::cell::RefCell;
use sails_rs::prelude::*;

pub mod notarizer;
pub mod types;

use notarizer::{NotarizerService, NotarizerState};

pub struct Program {
    notarizer: RefCell<NotarizerState>,
}

#[sails_rs::program]
impl Program {
    /// Construct the Notarizer program.
    /// `owner` is the wallet that can change fees and withdraw funds.
    pub fn new(owner: ActorId) -> Self {
        let mut state = NotarizerState::default();
        state.config.owner = owner;

        Self {
            notarizer: RefCell::new(state),
        }
    }

    pub fn notarizer(&self) -> NotarizerService<'_> {
        NotarizerService::new(&self.notarizer)
    }
}
