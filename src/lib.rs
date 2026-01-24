pub mod client;
pub mod error;
mod marc;
pub mod pdu;

pub use client::Client;
pub use error::{Error, Result};
pub use marc::{MarcRecord, parse_record, parse_records};

// Re-export commonly used PDU types and enums
pub use pdu::{
    // Core types
    Apdu, Credentials, Query, RpnQuery,
    // Init
    InitRequest, InitResponse,
    // Search
    SearchRequest, SearchResponse,
    // Present
    PresentRequest, PresentResponse, PresentStatus, Records,
    // Delete Result Set
    DeleteResultSetRequest, DeleteResultSetResponse, DeleteFunction, DeleteOperationStatus,
    // Scan
    ScanRequest, ScanResponse, ScanStatus, ListEntries, Entry, TermInfo,
    // Sort
    SortRequest, SortResponse, SortStatus, SortKeySpec, SortKey, SortElement,
    SortRelation, CaseSensitivity, MissingValueAction,
    // Close
    Close, CloseReason,
    // Extended Services
    ExtendedServicesRequest, ExtendedServicesResponse, ExtendedServicesFunction,
    ExtendedServicesStatus, WaitAction,
    // Resource Control
    ResourceControlRequest, ResourceControlResponse, TriggerResourceControlRequest,
    TriggerRequestedAction, PartialResultsAvailable,
    // Resource Report
    ResourceReportRequest, ResourceReportResponse, ResourceReportStatus,
    // Access Control
    AccessControlRequest, AccessControlResponse,
    // Duplicate Detection
    DuplicateDetectionRequest, DuplicateDetectionResponse, DuplicateDetectionStatus,
    // Segment
    Segment,
    // Helper functions
    bib1_attribute_set, record_syntax_usmarc, make_init_request, make_type1_query,
    make_search_request, make_present_request, make_delete_result_set_request,
    make_delete_all_result_sets_request, make_scan_request, make_sort_request,
    make_sort_key_by_field, make_close_request, make_extended_services_request,
    make_duplicate_detection_request, make_resource_control_response,
    make_access_control_response, make_trigger_resource_control_request,
    make_resource_report_request, extract_marc_records,
};
