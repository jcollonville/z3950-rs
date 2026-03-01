use crate::error::{Error, Result};
use marc_rs::{parse_auto, Encoding, FormatEncoding, MarcFormat, Record};

pub type MarcRecord = Record;

fn marc21_utf8() -> FormatEncoding {
    FormatEncoding::new(MarcFormat::Marc21, Encoding::Utf8)
}

/// Parses a single raw MARC record (MARC21) into a `marc_rs::Record`.
pub fn parse_record(raw: &[u8]) -> Result<MarcRecord> {
    let records = parse_auto(raw, None).map_err(|e| Error::Marc(e.to_string()))?;
    records.records.into_iter().next().ok_or_else(|| Error::Marc("Empty MARC record".into()))
}

/// Parses a collection of raw MARC records.
pub fn parse_records(raw_records: &[Vec<u8>]) -> Result<Vec<MarcRecord>> {
    let mut output = Vec::new();
    for raw in raw_records {
        let mut parsed = parse_auto(raw, None).map_err(|e| Error::Marc(e.to_string()))?;
        output.append(&mut parsed.records);
    }
    Ok(output)
}
