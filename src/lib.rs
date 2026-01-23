pub mod client;
pub mod error;
mod marc;
pub mod pdu;

pub use client::Client;
pub use error::{Error, Result};
pub use marc::{MarcRecord, parse_record, parse_records};
