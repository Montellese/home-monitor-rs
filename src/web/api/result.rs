use rocket::http::Status;

pub type Result<T> = std::result::Result<T, (Status, String)>;

pub fn handle<T, E: std::fmt::Display>(
    result: std::result::Result<T, E>,
    status: Status,
) -> Result<T> {
    match result {
        Ok(value) => Ok(value),
        Err(e) => Err((status, e.to_string())),
    }
}

pub fn handle_internal_server_error<T, E: std::fmt::Display>(
    result: std::result::Result<T, E>,
) -> Result<T> {
    handle(result, Status::InternalServerError)
}

pub fn handle_not_found<T, E: std::fmt::Display>(result: std::result::Result<T, E>) -> Result<T> {
    handle(result, Status::NotFound)
}
