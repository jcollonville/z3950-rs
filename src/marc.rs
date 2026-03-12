use crate::error::{Error, Result};

use marc_rs::MarcReader;

pub type MarcRecord = marc_rs::Record;

/// Parses a collection of raw MARC records.
pub fn parse_records(raw_records: Vec<u8>) -> Result<Vec<MarcRecord>> {
    let reader = MarcReader::from_bytes(raw_records).map_err(|e| Error::Marc(e.to_string()))?;
    reader.into_records().map_err(|e| Error::Marc(e.to_string()))
}
