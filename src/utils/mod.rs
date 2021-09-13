mod always_on;
mod always_on_file;

pub use always_on::AlwaysOn;
#[cfg(test)]
pub use always_on::MockAlwaysOn;
pub use always_on_file::AlwaysOnFile;

#[cfg(test)]
pub use sn_fake_clock::FakeClock as Instant;
#[cfg(not(test))]
pub use std::time::Instant;
