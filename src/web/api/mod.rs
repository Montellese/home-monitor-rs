mod config;
pub mod result;
pub mod server;
mod status;

pub use config::get_config;
pub use status::get_status;
#[cfg(test)]
pub use status::Status;
