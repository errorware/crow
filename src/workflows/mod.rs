//! Workflow tests: Crow opened headless in a test window and driven the way
//! a person would, through the same methods its buttons call, against real
//! servers. The plans and results are in `workflows/` (not in git).
//!
//! `CROW_WORKFLOWS=<scratch dir> cargo test workflows -- --test-threads=1 --nocapture`
//!
//! Without CROW_WORKFLOWS every test returns at once, so `cargo test` stays
//! offline. They share one profile and run in order (w01, w02, ...).

#[cfg(all(test, unix))]
mod harness;
#[cfg(all(test, unix))]
mod w00_reset;
#[cfg(all(test, unix))]
mod w01_enroll;
#[cfg(all(test, unix))]
mod w02_bastion;
#[cfg(all(test, unix))]
mod w03_push_configs;
#[cfg(all(test, unix))]
mod w04_detach_bastion;
#[cfg(all(test, unix))]
mod w05_users;
#[cfg(all(test, unix))]
mod w06_hardening;
#[cfg(all(test, unix))]
mod w07_key_rotation;
#[cfg(all(test, unix))]
mod w08_host_keys;
#[cfg(all(test, unix))]
mod w09_drift;
#[cfg(all(test, unix))]
mod w10_dokku;
