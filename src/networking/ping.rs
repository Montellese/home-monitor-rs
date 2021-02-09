use super::networking_error::NetworkingError;

pub fn ping(ip: &str) -> Result<bool, NetworkingError> {
    Err(NetworkingError("ping not implemented!".to_string()))
}