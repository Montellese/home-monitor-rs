mod always_off;
mod always_off_file;
mod always_on;
mod always_on_file;

#[cfg(not(test))]
pub use std::time::Instant;

pub use always_off::AlwaysOff;
#[cfg(test)]
pub use always_off::MockAlwaysOff;
pub use always_off_file::AlwaysOffFile;
pub use always_on::AlwaysOn;
#[cfg(test)]
pub use always_on::MockAlwaysOn;
pub use always_on_file::AlwaysOnFile;
#[cfg(test)]
pub use sn_fake_clock::FakeClock as Instant;
