pub mod client;
pub mod error;
mod marc;
pub mod pdu;
pub mod query_languages;
pub use query_languages::QueryLanguage;
pub use client::Client;
pub use error::{Error, Result};
pub use marc::{parse_record, parse_records, MarcRecord};

// Re-export commonly used PDU types and enums
pub use pdu::{
    // Helper functions
    bib1_attribute_set,
    extract_marc_records,
    make_access_control_response,
    make_close_request,
    make_delete_all_result_sets_request,
    make_delete_result_set_request,
    make_duplicate_detection_request,
    make_extended_services_request,
    make_init_request,
    make_present_request,
    make_resource_control_response,
    make_resource_report_request,
    make_scan_request,
    make_search_request,
    make_sort_key_by_field,
    make_sort_request,
    make_trigger_resource_control_request,
    make_type1_query,
    record_syntax_usmarc,
    // Access Control
    AccessControlRequest,
    AccessControlResponse,
    // Core types
    Apdu,
    CaseSensitivity,
    // Close
    Close,
    CloseReason,
    Credentials,
    DeleteFunction,
    DeleteOperationStatus,
    // Delete Result Set
    DeleteResultSetRequest,
    DeleteResultSetResponse,
    // Duplicate Detection
    DuplicateDetectionRequest,
    DuplicateDetectionResponse,
    DuplicateDetectionStatus,
    Entry,
    ExtendedServicesFunction,
    // Extended Services
    ExtendedServicesRequest,
    ExtendedServicesResponse,
    ExtendedServicesStatus,
    // Init
    InitRequest,
    InitResponse,
    ListEntries,
    MissingValueAction,
    PartialResultsAvailable,
    // Present
    PresentRequest,
    PresentResponse,
    PresentStatus,
    Query,
    Records,
    // Resource Control
    ResourceControlRequest,
    ResourceControlResponse,
    // Resource Report
    ResourceReportRequest,
    ResourceReportResponse,
    ResourceReportStatus,
    RpnQuery,
    // Scan
    ScanRequest,
    ScanResponse,
    ScanStatus,
    // Search
    SearchRequest,
    SearchResponse,
    // Segment
    Segment,
    SortElement,
    SortKey,
    SortKeySpec,
    SortRelation,
    // Sort
    SortRequest,
    SortResponse,
    SortStatus,
    TermInfo,
    TriggerRequestedAction,
    TriggerResourceControlRequest,
    WaitAction,
};
