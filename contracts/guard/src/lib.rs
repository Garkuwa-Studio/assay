//! Assay Guard: `is_healthy(asset, policy)` circuit breaker. Never moves funds.
//! Placeholder — no functionality yet.
#![no_std]

use soroban_sdk::{contract, contractimpl};

#[contract]
pub struct Guard;

#[contractimpl]
impl Guard {
    /// Placeholder. Returns the contract interface version.
    pub fn version() -> u32 {
        0
    }
}

#[cfg(test)]
mod test;
