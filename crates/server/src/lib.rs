//! Shared operator marketplace and server cartridge-catalog services.

pub mod cartridge_catalog;
pub mod cartridge_distribution;
pub mod marketplace_egress;
pub mod marketplace_sync;
pub mod session_cartridges;

#[cfg(test)]
mod marketplace_sync_tests;
