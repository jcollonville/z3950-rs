use crate::error::{Error, Result};
use crate::marc::{MarcRecord, parse_records};
use crate::pdu::{Apdu, Credentials, SearchResponse, extract_marc_records, make_init_request, make_present_request, make_search_request, make_type1_query};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

const DEFAULT_RESULT_SET: &str = "default";

pub struct Client {
    stream: TcpStream,
    result_set: String,
}

impl Client {
    /// Connects to a Z39.50 target and performs an Init handshake.
    pub async fn connect(addr: &str) -> Result<Self> {
        Self::connect_with_credentials(addr, None).await
    }

    /// Connects to a Z39.50 target and performs an Init handshake using simple credentials.
    pub async fn connect_with_credentials(addr: &str, credentials: Option<(&str, &str)>) -> Result<Self> {
        let mut stream = TcpStream::connect(addr).await?;

        let credentials = credentials.map(|(u, p)| Credentials::new(u, p));
        let init = make_init_request(credentials.as_ref())?;
        send_pdu(&mut stream, &Apdu::InitRequest(init)).await?;

        let _r = read_pdu::<crate::pdu::InitResponse>(&mut stream).await?;

        Ok(Self {
            stream,
            result_set: DEFAULT_RESULT_SET.to_string(),
        })
    }

    /// Executes a Type-1 (RPN) search against the given databases.
    pub async fn search(&mut self, databases: &[&str], term: &str) -> Result<SearchResponse> {
        let dbs: Vec<String> = databases.iter().map(|s| s.to_string()).collect();
        let query = make_type1_query(4, term)?;
        let req = make_search_request(&dbs, &self.result_set, query)?;
        send_pdu(&mut self.stream, &Apdu::SearchRequest(req)).await?;

        let r = read_pdu::<SearchResponse>(&mut self.stream).await?;
        Ok(r)
    }

    pub async fn present_raw(&mut self, start: i64, count: i64) -> Result<Vec<Vec<u8>>> {
        let req = make_present_request(&self.result_set, start, count)?;
        send_pdu(&mut self.stream, &Apdu::PresentRequest(req)).await?;
        let r = read_pdu::<crate::pdu::PresentResponse>(&mut self.stream).await?;
        extract_marc_records(&r)
    }

    pub async fn present_marc(&mut self, start: i64, count: i64) -> Result<Vec<MarcRecord>> {
        let raw = self.present_raw(start, count).await?;
        parse_records(&raw)
    }

    pub async fn scan(&mut self, _database: &str, _term: &str, _count: i64) -> Result<()> {
        Err(Error::Protocol("Scan operation not implemented in this preview".into()))
    }
}

async fn send_pdu(stream: &mut TcpStream, pdu: &Apdu) -> Result<()> {
    let bytes = rasn::ber::encode(pdu).map_err(|e| Error::BerEncode(e.to_string()))?;

    stream.write_all(&bytes).await?;
    stream.flush().await?;
    Ok(())
}

async fn read_pdu<T: rasn::AsnType + rasn::Decode>(stream: &mut TcpStream) -> Result<T> {
    let frame = read_ber_frame(stream).await?;
    rasn::ber::decode::<T>(&frame).map_err(|e| Error::BerDecode(e.to_string()))
}

/// Reads a complete BER-encoded frame from the stream.
/// Handles both definite and indefinite length encodings.
async fn read_ber_frame(stream: &mut TcpStream) -> Result<Vec<u8>> {
    // Defensive cap to avoid allocating unbounded memory from potentially hostile inputs.
    const MAX_FRAME_SIZE: usize = 16 * 1024 * 1024; // 16 MiB
    let mut buffer = Vec::new();

    // 1. Read the tag (identifier octets)
    let first_byte = read_byte(stream).await?;
    buffer.push(first_byte);

    // High-tag-number form: if bits 1-5 are all 1s (0x1F), tag continues
    if (first_byte & 0x1F) == 0x1F {
        loop {
            let b = read_byte(stream).await?;
            buffer.push(b);
            // Last octet has bit 8 = 0
            if (b & 0x80) == 0 {
                break;
            }
        }
    }

    // 2. Read the length octet(s)
    let len_byte = read_byte(stream).await?;
    buffer.push(len_byte);

    if len_byte == 0x80 {
        // Indefinite length: read until we find the terminating 00 00
        // We need to track nested indefinite lengths
        read_indefinite_contents(stream, &mut buffer, MAX_FRAME_SIZE).await?;
    } else if (len_byte & 0x80) != 0 {
        // Long form: bits 1-7 indicate number of subsequent length octets
        let num_len_octets = (len_byte & 0x7F) as usize;
        if num_len_octets == 0 || num_len_octets > 8 {
            return Err(Error::BerDecode(format!("Invalid BER length: {} octets", num_len_octets)));
        }

        let mut length: usize = 0;
        for _ in 0..num_len_octets {
            let b = read_byte(stream).await?;
            buffer.push(b);
            length = (length << 8) | (b as usize);
        }

        // Read the definite content
        if buffer.len().saturating_add(length) > MAX_FRAME_SIZE {
            return Err(Error::FrameTooLarge { max: MAX_FRAME_SIZE });
        }
        let mut content = vec![0u8; length];
        stream.read_exact(&mut content).await?;
        buffer.extend(content);
    } else {
        // Short form: length is directly in bits 1-7
        let length = len_byte as usize;
        if length > 0 {
            if buffer.len().saturating_add(length) > MAX_FRAME_SIZE {
                return Err(Error::FrameTooLarge { max: MAX_FRAME_SIZE });
            }
            let mut content = vec![0u8; length];
            stream.read_exact(&mut content).await?;
            buffer.extend(content);
        }
    }

    Ok(buffer)
}

/// Reads content with indefinite length encoding until end-of-contents (00 00).
/// Handles nested constructed types with indefinite length using an iterative approach.
async fn read_indefinite_contents(stream: &mut TcpStream, buffer: &mut Vec<u8>, max_frame_size: usize) -> Result<()> {
    // Track nesting depth: we start at depth 1 (for the initial indefinite-length container)
    let mut depth: usize = 1;

    while depth > 0 {
        // Read the tag byte
        let first = read_byte(stream).await?;
        buffer.push(first);
        if buffer.len() > max_frame_size {
            return Err(Error::FrameTooLarge { max: max_frame_size });
        }

        if first == 0x00 {
            let second = read_byte(stream).await?;
            buffer.push(second);
            if second == 0x00 {
                // End-of-contents marker: close one nesting level
                depth -= 1;
                continue;
            }
            // Not end-of-contents, 0x00 was a tag (unusual but valid)
            // The second byte is the length, handle it
            if second == 0x80 {
                // Nested indefinite length
                depth += 1;
            } else if (second & 0x80) != 0 {
                // Long form length
                let num_len_octets = (second & 0x7F) as usize;
                let mut length: usize = 0;
                for _ in 0..num_len_octets {
                    let b = read_byte(stream).await?;
                    buffer.push(b);
                    length = (length << 8) | (b as usize);
                }
                if buffer.len().saturating_add(length) > max_frame_size {
                    return Err(Error::FrameTooLarge { max: max_frame_size });
                }
                let mut content = vec![0u8; length];
                stream.read_exact(&mut content).await?;
                buffer.extend(content);
            } else {
                // Short form length
                let length = second as usize;
                if length > 0 {
                    if buffer.len().saturating_add(length) > max_frame_size {
                        return Err(Error::FrameTooLarge { max: max_frame_size });
                    }
                    let mut content = vec![0u8; length];
                    stream.read_exact(&mut content).await?;
                    buffer.extend(content);
                }
            }
            continue;
        }

        // High-tag-number form
        if (first & 0x1F) == 0x1F {
            loop {
                let b = read_byte(stream).await?;
                buffer.push(b);
                if (b & 0x80) == 0 {
                    break;
                }
            }
        }

        // Read length
        let len_byte = read_byte(stream).await?;
        buffer.push(len_byte);

        if len_byte == 0x80 {
            // Nested indefinite length: increase depth
            depth += 1;
        } else if (len_byte & 0x80) != 0 {
            // Long form
            let num_len_octets = (len_byte & 0x7F) as usize;
            let mut length: usize = 0;
            for _ in 0..num_len_octets {
                let b = read_byte(stream).await?;
                buffer.push(b);
                length = (length << 8) | (b as usize);
            }
            if buffer.len().saturating_add(length) > max_frame_size {
                return Err(Error::FrameTooLarge { max: max_frame_size });
            }
            let mut content = vec![0u8; length];
            stream.read_exact(&mut content).await?;
            buffer.extend(content);
        } else {
            // Short form
            let length = len_byte as usize;
            if length > 0 {
                if buffer.len().saturating_add(length) > max_frame_size {
                    return Err(Error::FrameTooLarge { max: max_frame_size });
                }
                let mut content = vec![0u8; length];
                stream.read_exact(&mut content).await?;
                buffer.extend(content);
            }
        }
    }

    Ok(())
}

/// Reads a single byte from the stream.
async fn read_byte(stream: &mut TcpStream) -> Result<u8> {
    let mut buf = [0u8; 1];
    stream.read_exact(&mut buf).await?;
    Ok(buf[0])
}
