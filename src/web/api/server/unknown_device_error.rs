use std::fmt;
use std::io::Cursor;

use rocket::response::Responder;
use rocket::{http, response, Request, Response};
use rocket_okapi::gen::OpenApiGenerator;
use rocket_okapi::okapi::openapi3::Responses;
use rocket_okapi::response::OpenApiResponderInner;

use crate::dom::DeviceId;

#[derive(Debug)]
pub struct UnknownDeviceError(DeviceId);

impl std::error::Error for UnknownDeviceError {}

impl From<DeviceId> for UnknownDeviceError {
    fn from(device_id: DeviceId) -> Self {
        Self(device_id)
    }
}

impl From<&DeviceId> for UnknownDeviceError {
    fn from(device_id: &DeviceId) -> Self {
        Self::from(device_id.clone())
    }
}

impl fmt::Display for UnknownDeviceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[UnknownDeviceError] {}", self.0)
    }
}

impl<'r, 'o: 'r> Responder<'r, 'o> for UnknownDeviceError {
    fn respond_to(self, _: &Request) -> response::Result<'o> {
        let error_msg = self.to_string();
        Response::build()
            .header(http::ContentType::Plain)
            .status(http::Status::NotFound)
            .sized_body(error_msg.len(), Cursor::new(error_msg))
            .ok()
    }
}

impl OpenApiResponderInner for UnknownDeviceError {
    fn responses(_: &mut OpenApiGenerator) -> rocket_okapi::Result<Responses> {
        let mut responses = Responses::default();
        add_404_error(&mut responses);
        Ok(responses)
    }
}

fn add_404_error(responses: &mut Responses) {
    responses.responses.entry("404".to_owned())
        .or_insert_with(|| {
            let response = rocket_okapi::okapi::openapi3::Response{
                description: "\
                    [404 Not Found](https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/404)\n\n\
                    This response is given when you request a page that does not exists.\n\n\
                    **Note:** This is not exactly a response by this endpoint. But might be returned when you wrongly \
                    input one or more of the path or query parameters. An example would be that you have provided an \
                    unknown server.\n\n\
                    So when you get this error and you expect a result. Check all the types of the parameters. \
                    ".to_owned(),
                ..Default::default()
            };
            response.into()
        });
}
