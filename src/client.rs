use crate::error::{Error, Result};
use crate::marc::{parse_records, MarcRecord};
use crate::pdu::{
    extract_marc_records, make_access_control_response, make_close_request, make_delete_all_result_sets_request, make_delete_result_set_request, make_duplicate_detection_request,
    make_extended_services_request, make_init_request, make_present_request, make_resource_control_response, make_resource_report_request, make_scan_request, make_search_request,
    make_sort_key_by_field, make_sort_request, make_trigger_resource_control_request, make_type1_query, Apdu, Close, CloseReason, Credentials, DeleteOperationStatus, DeleteResultSetResponse,
    DuplicateDetectionResponse, DuplicateDetectionStatus, ExtendedServicesFunction, ExtendedServicesResponse, ExtendedServicesStatus, External, ListEntries, ResourceReport, ResourceReportResponse,
    ResourceReportStatus, ScanResponse, ScanStatus, SearchResponse, SortKeySpec, SortResponse, SortStatus, TriggerRequestedAction, WaitAction,
};
use rasn::types::ObjectIdentifier;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

const DEFAULT_RESULT_SET: &str = "default";

pub struct Client {
    stream: TcpStream,
    result_set: String,
    closed: bool,
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
            closed: false,
        })
    }

    /// Returns the current result set name.
    pub fn result_set_name(&self) -> &str {
        &self.result_set
    }

    /// Sets a custom result set name for subsequent operations.
    pub fn set_result_set_name(&mut self, name: impl Into<String>) {
        self.result_set = name.into();
    }

    /// Executes a Type-1 (RPN) search against the given databases.
    pub async fn search(&mut self, databases: &[&str], term: &str) -> Result<SearchResponse> {
        self.check_not_closed()?;
        let dbs: Vec<String> = databases.iter().map(|s| s.to_string()).collect();
        let query = make_type1_query(4, term)?;
        let req = make_search_request(&dbs, &self.result_set, query)?;
        send_pdu(&mut self.stream, &Apdu::SearchRequest(req)).await?;

        let r = read_pdu::<SearchResponse>(&mut self.stream).await?;
        Ok(r)
    }

    /// Presents raw record data from the current result set.
    pub async fn present_raw(&mut self, start: i64, count: i64) -> Result<Vec<Vec<u8>>> {
        self.check_not_closed()?;
        let req = make_present_request(&self.result_set, start, count)?;
        send_pdu(&mut self.stream, &Apdu::PresentRequest(req)).await?;
        let r = read_pdu::<crate::pdu::PresentResponse>(&mut self.stream).await?;
        extract_marc_records(&r)
    }

    /// Presents and parses MARC records from the current result set.
    pub async fn present_marc(&mut self, start: i64, count: i64) -> Result<Vec<MarcRecord>> {
        let raw = self.present_raw(start, count).await?;
        parse_records(&raw)
    }

    // ========================================================================
    // Delete Result Set
    // ========================================================================

    /// Deletes specific result sets by name.
    pub async fn delete_result_sets(&mut self, result_sets: &[&str]) -> Result<DeleteResultSetResponse> {
        self.check_not_closed()?;
        let req = make_delete_result_set_request(result_sets);
        send_pdu(&mut self.stream, &Apdu::DeleteResultSetRequest(req)).await?;
        let r = read_pdu::<DeleteResultSetResponse>(&mut self.stream).await?;
        Ok(r)
    }

    /// Deletes all result sets.
    pub async fn delete_all_result_sets(&mut self) -> Result<DeleteResultSetResponse> {
        self.check_not_closed()?;
        let req = make_delete_all_result_sets_request();
        send_pdu(&mut self.stream, &Apdu::DeleteResultSetRequest(req)).await?;
        let r = read_pdu::<DeleteResultSetResponse>(&mut self.stream).await?;
        Ok(r)
    }

    /// Returns true if the delete operation was successful.
    pub fn delete_was_successful(response: &DeleteResultSetResponse) -> bool {
        response.delete_operation_status == DeleteOperationStatus::Success
    }

    // ========================================================================
    // Scan
    // ========================================================================

    /// Scans (browses) an index starting from the given term.
    ///
    /// # Arguments
    /// * `databases` - Database names to scan
    /// * `term` - Starting term for the scan
    /// * `attribute_type` - BIB-1 attribute type (e.g., 4 for title, 1 for author)
    /// * `count` - Number of terms to retrieve
    /// * `preferred_position` - Preferred position of the starting term in results
    pub async fn scan(&mut self, databases: &[&str], term: &str, attribute_type: i64, count: i64, preferred_position: Option<i64>) -> Result<ScanResponse> {
        self.check_not_closed()?;
        let dbs: Vec<String> = databases.iter().map(|s| s.to_string()).collect();
        let req = make_scan_request(&dbs, term, attribute_type, None, count, preferred_position)?;

        send_pdu(&mut self.stream, &Apdu::ScanRequest(req)).await?;

        let r = read_pdu::<ScanResponse>(&mut self.stream).await?;

        Ok(r)
    }

    /// Returns true if the scan was successful.
    pub fn scan_was_successful(response: &ScanResponse) -> bool {
        response.scan_status == ScanStatus::Success
    }

    /// Extracts term entries from a scan response.
    pub fn extract_scan_entries(response: &ScanResponse) -> Option<&ListEntries> {
        response.entries.as_ref()
    }

    // ========================================================================
    // Sort
    // ========================================================================

    /// Sorts one or more result sets into a new result set.
    ///
    /// # Arguments
    /// * `input_result_sets` - Names of input result sets
    /// * `output_result_set` - Name for the sorted output result set
    /// * `sort_keys` - Sort specifications (use `make_sort_key_by_field` helper)
    pub async fn sort(&mut self, input_result_sets: &[&str], output_result_set: &str, sort_keys: Vec<SortKeySpec>) -> Result<SortResponse> {
        self.check_not_closed()?;
        let req = make_sort_request(input_result_sets, output_result_set, sort_keys);
        send_pdu(&mut self.stream, &Apdu::SortRequest(req)).await?;
        let r = read_pdu::<SortResponse>(&mut self.stream).await?;
        Ok(r)
    }

    /// Creates a sort key for sorting by a field name.
    pub fn sort_key_by_field(field_name: &str, ascending: bool, case_sensitive: bool) -> SortKeySpec {
        make_sort_key_by_field(field_name, ascending, case_sensitive)
    }

    /// Returns true if the sort was successful.
    pub fn sort_was_successful(response: &SortResponse) -> bool {
        response.sort_status == SortStatus::Success
    }

    // ========================================================================
    // Close
    // ========================================================================

    /// Closes the Z39.50 session gracefully.
    pub async fn close(&mut self) -> Result<Close> {
        self.close_with_reason(CloseReason::Finished, None).await
    }

    /// Closes the Z39.50 session with a specific reason.
    pub async fn close_with_reason(&mut self, reason: CloseReason, diagnostic_info: Option<&str>) -> Result<Close> {
        if self.closed {
            return Err(Error::Protocol("Connection already closed".into()));
        }
        let req = make_close_request(reason, diagnostic_info);
        send_pdu(&mut self.stream, &Apdu::Close(req)).await?;
        let r = read_pdu::<Close>(&mut self.stream).await?;
        self.closed = true;
        Ok(r)
    }

    /// Returns true if the connection has been closed.
    pub fn is_closed(&self) -> bool {
        self.closed
    }

    // ========================================================================
    // Extended Services
    // ========================================================================

    /// Sends an extended services request.
    ///
    /// # Arguments
    /// * `function` - Create, Delete, or Modify
    /// * `package_type` - OID identifying the type of extended service
    /// * `package_name` - Optional name for the task package
    /// * `task_specific_parameters` - Optional externally-defined parameters
    /// * `wait_action` - How to handle waiting for completion
    pub async fn extended_services(
        &mut self,
        function: ExtendedServicesFunction,
        package_type: ObjectIdentifier,
        package_name: Option<&str>,
        task_specific_parameters: Option<External>,
        wait_action: WaitAction,
    ) -> Result<ExtendedServicesResponse> {
        self.check_not_closed()?;
        let req = make_extended_services_request(function, package_type, package_name, task_specific_parameters, wait_action);
        send_pdu(&mut self.stream, &Apdu::ExtendedServicesRequest(req)).await?;
        let r = read_pdu::<ExtendedServicesResponse>(&mut self.stream).await?;
        Ok(r)
    }

    /// Returns true if the extended services operation completed successfully.
    pub fn extended_services_was_successful(response: &ExtendedServicesResponse) -> bool {
        matches!(response.operation_status, ExtendedServicesStatus::Done | ExtendedServicesStatus::Accepted)
    }

    // ========================================================================
    // Duplicate Detection
    // ========================================================================

    /// Performs duplicate detection across result sets.
    ///
    /// # Arguments
    /// * `input_result_sets` - Names of input result sets to check for duplicates
    /// * `output_result_set` - Name for the output result set
    /// * `clustering` - Whether to cluster duplicates together
    pub async fn duplicate_detection(&mut self, input_result_sets: &[&str], output_result_set: &str, clustering: bool) -> Result<DuplicateDetectionResponse> {
        self.check_not_closed()?;
        let req = make_duplicate_detection_request(input_result_sets, output_result_set, clustering);
        send_pdu(&mut self.stream, &Apdu::DuplicateDetectionRequest(req)).await?;
        let r = read_pdu::<DuplicateDetectionResponse>(&mut self.stream).await?;
        Ok(r)
    }

    /// Returns true if duplicate detection was successful.
    pub fn duplicate_detection_was_successful(response: &DuplicateDetectionResponse) -> bool {
        response.status == DuplicateDetectionStatus::Success
    }

    // ========================================================================
    // Resource Control
    // ========================================================================

    /// Sends a resource control response (typically in response to a resource control request from the server).
    pub async fn send_resource_control_response(&mut self, continue_flag: bool, result_set_wanted: Option<bool>) -> Result<()> {
        self.check_not_closed()?;
        let resp = make_resource_control_response(continue_flag, result_set_wanted);
        send_pdu(&mut self.stream, &Apdu::ResourceControlResponse(resp)).await?;
        Ok(())
    }

    /// Triggers a resource control action on the server.
    pub async fn trigger_resource_control(&mut self, action: TriggerRequestedAction, result_set_wanted: Option<bool>) -> Result<()> {
        self.check_not_closed()?;
        let req = make_trigger_resource_control_request(action, result_set_wanted);
        send_pdu(&mut self.stream, &Apdu::TriggerResourceControlRequest(req)).await?;
        Ok(())
    }

    // ========================================================================
    // Resource Report
    // ========================================================================

    /// Requests a resource report from the server.
    pub async fn resource_report(&mut self, op_id: Option<&[u8]>, preferred_format: Option<ObjectIdentifier>) -> Result<ResourceReportResponse> {
        self.check_not_closed()?;
        let req = make_resource_report_request(op_id, preferred_format);
        send_pdu(&mut self.stream, &Apdu::ResourceReportRequest(req)).await?;
        let r = read_pdu::<ResourceReportResponse>(&mut self.stream).await?;
        Ok(r)
    }

    /// Returns true if the resource report request was successful.
    pub fn resource_report_was_successful(response: &ResourceReportResponse) -> bool {
        response.resource_report_status == ResourceReportStatus::Success
    }

    /// Extracts the resource report from the response.
    pub fn extract_resource_report(response: &ResourceReportResponse) -> Option<&ResourceReport> {
        response.resource_report.as_ref()
    }

    // ========================================================================
    // Access Control
    // ========================================================================

    /// Sends an access control response (typically in response to an access control request from the server).
    pub async fn send_access_control_response(&mut self, response_data: &[u8]) -> Result<()> {
        self.check_not_closed()?;
        let resp = make_access_control_response(response_data);
        send_pdu(&mut self.stream, &Apdu::AccessControlResponse(resp)).await?;
        Ok(())
    }

    // ========================================================================
    // Internal helpers
    // ========================================================================

    fn check_not_closed(&self) -> Result<()> {
        if self.closed {
            Err(Error::Protocol("Connection is closed".into()))
        } else {
            Ok(())
        }
    }
}

async fn send_pdu(stream: &mut TcpStream, pdu: &Apdu) -> Result<()> {
    let bytes = rasn::ber::encode(pdu).map_err(|e| Error::BerEncode(e.to_string()))?;

    stream.write_all(&bytes).await?;
    stream.flush().await?;
    Ok(())
}

async fn read_pdu<T: rasn::AsnType + rasn::Decode + std::fmt::Debug>(stream: &mut TcpStream) -> Result<T> {
    let frame = read_ber_frame(stream).await?;

    let decoded = rasn::ber::decode::<T>(&frame).map_err(|e| Error::BerDecode(e.to_string()))?;
    Ok(decoded)
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
