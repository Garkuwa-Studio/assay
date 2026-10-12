//! TESTNET DEMO ONLY. Not a real lending pool; never deploy to mainnet.
//! Exists only to demonstrate Assay Guard on testnet. Placeholder.
#![no_std]

use soroban_sdk::{contract, contractimpl};

#[contract]
pub struct DemoPool;

#[contractimpl]
impl DemoPool {
    /// Placeholder. Returns the contract interface version.
    pub fn version() -> u32 {
        0
    }
}

#[cfg(test)]
mod test;
