//! Assay reserve registry: attestors post signed reserve reports per asset.
//! Placeholder — no functionality yet.
#![no_std]

use soroban_sdk::{contract, contractimpl};

#[contract]
pub struct Registry;

#[contractimpl]
impl Registry {
    /// Placeholder. Returns the contract interface version.
    pub fn version() -> u32 {
        0
    }
}

#[cfg(test)]
mod test;
