use crate::utils::consts::BINARY_PROTOCOL_VERSION;

pub mod socket;

#[rocket::get("/binary-protocol-version")]
pub fn binary_protocol_version_endpoint() -> rocket::serde::json::Json<u8> {
    rocket::serde::json::Json(BINARY_PROTOCOL_VERSION)
}
