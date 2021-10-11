use std::fmt;

use rocket::response::Responder;
use rocket::{response, Request};
use rocket_okapi::gen::OpenApiGenerator;
use rocket_okapi::okapi::openapi3::Responses;
use rocket_okapi::response::OpenApiResponderInner;

use crate::web::api::server::UnknownDeviceError;
use crate::web::api::InternalServerError;

#[derive(Debug)]
pub enum Error {
    UnknownDevice(UnknownDeviceError),
    Internal(InternalServerError),
}

impl std::error::Error for Error {}

impl From<UnknownDeviceError> for Error {
    fn from(error: UnknownDeviceError) -> Self {
        Self::UnknownDevice(error)
    }
}

impl From<InternalServerError> for Error {
    fn from(error: InternalServerError) -> Self {
        Self::Internal(error)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownDevice(error) => error.fmt(f),
            Self::Internal(error) => error.fmt(f),
        }
    }
}

impl<'r, 'o: 'r> Responder<'r, 'o> for Error {
    fn respond_to(self, req: &Request) -> response::Result<'o> {
        match self {
            Self::UnknownDevice(error) => error.respond_to(req),
            Self::Internal(error) => error.respond_to(req),
        }
    }
}

impl OpenApiResponderInner for Error {
    fn responses(gen: &mut OpenApiGenerator) -> rocket_okapi::Result<Responses> {
        let mut responses = Responses::default();
        {
            let responses_unknown_device = UnknownDeviceError::responses(gen)?;
            responses
                .responses
                .extend(responses_unknown_device.responses);
        }
        {
            let responses_internal_server_error = InternalServerError::responses(gen)?;
            responses
                .responses
                .extend(responses_internal_server_error.responses);
        }
        Ok(responses)
    }
}
