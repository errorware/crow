//! Workflow tests: Crow opened headless in a test window and driven the way
//! a person would, through the same methods its buttons call, against real
//! servers. The plans and results are in `workflows/` (not in git).
//!
//! `CROW_WORKFLOWS=<scratch dir> cargo test workflows -- --test-threads=1 --nocapture`
//!
//! Without CROW_WORKFLOWS every test returns at once, so `cargo test` stays
//! offline. They share one profile and run in order (w01, w02, ...).

#[cfg(test)]
mod harness;
#[cfg(test)]
mod w01_enroll;
#[cfg(test)]
mod w02_bastion;
