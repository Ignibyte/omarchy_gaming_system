//! Shared operator marketplace and server cartridge-catalog services.

pub mod cartridge_catalog;
pub mod marketplace_egress;
pub mod marketplace_sync;

#[cfg(test)]
mod marketplace_sync_tests;
