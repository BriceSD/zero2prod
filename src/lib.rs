//! lib.rs

pub mod configuration;
pub mod domain;
pub mod routes;
pub mod startup;
pub mod telemetry;
pub mod email_client;
pub mod authentication;

#[cfg(test)] extern crate proptest;
