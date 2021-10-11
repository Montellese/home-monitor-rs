use std::fmt;
use std::io::Cursor;

use rocket::response::Responder;
use rocket::{http, response, Request, Response};
use rocket_okapi::gen::OpenApiGenerator;
use rocket_okapi::okapi::openapi3::Responses;
use rocket_okapi::response::OpenApiResponderInner;

#[derive(Debug)]
pub struct InternalServerError {
    error: anyhow::Error,
}

impl std::error::Error for InternalServerError {}

impl From<anyhow::Error> for InternalServerError {
    fn from(error: anyhow::Error) -> Self {
        Self { error }
    }
}

impl fmt::Display for InternalServerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[InternalServerError] {}", self.error)
    }
}

impl<'r, 'o: 'r> Responder<'r, 'o> for InternalServerError {
    fn respond_to(self, _: &Request) -> response::Result<'o> {
        let error_msg = self.error.to_string();
        Response::build()
            .header(http::ContentType::Plain)
            .status(http::Status::InternalServerError)
            .sized_body(error_msg.len(), Cursor::new(error_msg))
            .ok()
    }
}

impl OpenApiResponderInner for InternalServerError {
    fn responses(_: &mut OpenApiGenerator) -> rocket_okapi::Result<Responses> {
        let mut responses = Responses::default();
        add_500_error(&mut responses);
        Ok(responses)
    }
}

fn add_500_error(responses: &mut Responses) {
    responses
        .responses
        .entry("500".to_owned())
        .or_insert_with(|| {
            let response = rocket_okapi::okapi::openapi3::Response {
                description: "\
                    [500 Internal Server Error](https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/500)\n\n\
                    This response is given when the server has an internal error that it could not recover from.\
                    ".to_owned(),
                ..Default::default()
            };
            response.into()
        });
}
