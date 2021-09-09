pub mod always_on;
pub mod always_on_file;

#[cfg(test)]
pub use sn_fake_clock::FakeClock as Instant;
#[cfg(not(test))]
pub use std::time::Instant;
