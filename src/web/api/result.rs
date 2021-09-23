use rocket::http::Status;

pub type Result<T> = std::result::Result<T, (Status, String)>;

pub fn handle<T, E: std::fmt::Display>(result: std::result::Result<T, E>) -> Result<T> {
    match result {
        Ok(value) => Ok(value),
        Err(e) => Err((Status::InternalServerError, e.to_string())),
    }
}
