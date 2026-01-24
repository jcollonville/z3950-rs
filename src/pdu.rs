use rasn::types::{Any, BitString, GeneralizedTime, Integer, ObjectIdentifier, OctetString, Utf8String, VisibleString};
use rasn::{AsnType, Decode, Decoder, Encode, Encoder};

use crate::error::{Error, Result};

#[derive(AsnType, Encode, Decode, Debug)]
#[rasn(choice)]
pub enum DatabaseName {
    #[rasn(tag(context, 105))] // C've qu'utilise yaz-client (9F 69)
    General(VisibleString),
}

/// Simple credentials container for Init authentication.
#[derive(Debug, Clone)]
pub struct Credentials {
    pub username: String,
    pub password: String,
}

impl Credentials {
    pub fn new(username: impl Into<String>, password: impl Into<String>) -> Self {
        Self {
            username: username.into(),
            password: password.into(),
        }
    }
}

#[derive(Debug, AsnType, Encode, Decode)]
#[rasn(choice)]
pub enum Apdu {
    #[rasn(tag(context, 20))]
    InitRequest(InitRequest),
    #[rasn(tag(context, 21))]
    InitResponse(InitResponse),
    #[rasn(tag(context, 22))]
    SearchRequest(SearchRequest),
    #[rasn(tag(context, 23))]
    SearchResponse(SearchResponse),
    #[rasn(tag(context, 24))]
    PresentRequest(PresentRequest),
    #[rasn(tag(context, 25))]
    PresentResponse(PresentResponse),
    #[rasn(tag(context, 26))]
    DeleteResultSetRequest(DeleteResultSetRequest),
    #[rasn(tag(context, 27))]
    DeleteResultSetResponse(DeleteResultSetResponse),
    #[rasn(tag(context, 28))]
    AccessControlRequest(AccessControlRequest),
    #[rasn(tag(context, 29))]
    AccessControlResponse(AccessControlResponse),
    #[rasn(tag(context, 30))]
    ResourceControlRequest(ResourceControlRequest),
    #[rasn(tag(context, 31))]
    ResourceControlResponse(ResourceControlResponse),
    #[rasn(tag(context, 32))]
    TriggerResourceControlRequest(TriggerResourceControlRequest),
    #[rasn(tag(context, 33))]
    ResourceReportRequest(ResourceReportRequest),
    #[rasn(tag(context, 34))]
    ResourceReportResponse(ResourceReportResponse),
    #[rasn(tag(context, 35))]
    ScanRequest(ScanRequest),
    #[rasn(tag(context, 36))]
    ScanResponse(ScanResponse),
    #[rasn(tag(context, 43))]
    SortRequest(SortRequest),
    #[rasn(tag(context, 44))]
    SortResponse(SortResponse),
    #[rasn(tag(context, 45))]
    Segment(Segment),
    #[rasn(tag(context, 46))]
    ExtendedServicesRequest(ExtendedServicesRequest),
    #[rasn(tag(context, 47))]
    ExtendedServicesResponse(ExtendedServicesResponse),
    #[rasn(tag(context, 48))]
    Close(Close),
    #[rasn(tag(context, 49))]
    DuplicateDetectionRequest(DuplicateDetectionRequest),
    #[rasn(tag(context, 50))]
    DuplicateDetectionResponse(DuplicateDetectionResponse),
}

#[derive(Debug, AsnType, Encode, Decode)]
#[rasn(tag(20))]
pub struct InitRequest {
    // 1. Toujours Optionnel
    #[rasn(tag(2))]
    pub reference_id: Option<OctetString>,

    // 2. OBLIGATOIRE (Tag 3)
    #[rasn(tag(3))]
    pub protocol_version: BitString,

    // 3. OBLIGATOIRE (Tag 4)
    #[rasn(tag(4))]
    pub options: BitString,

    // 4. OBLIGATOIRE (Tag 5)
    #[rasn(tag(5))]
    pub preferred_message_size: Integer,

    // 5. OBLIGATOIRE (Tag 6)
    #[rasn(tag(6))]
    pub exceptional_record_size: Integer,

    // 6. OPTIONNEL (Tag 7) - DOIT VENIR AVANT 110+
    #[rasn(tag(7))]
    pub id_authentication: Option<IdAuthentication>,

    // 7. OPTIONNELS (110+)
    #[rasn(tag(110))]
    pub implementation_id: Option<Utf8String>,
    #[rasn(tag(111))]
    pub implementation_name: Option<Utf8String>,
    #[rasn(tag(112))]
    pub implementation_version: Option<Utf8String>,

    #[rasn(tag(11))]
    pub user_information_field: Option<OctetString>,
    pub other_info: Option<OctetString>,
}

#[derive(Debug, AsnType, Encode, Decode)]
#[rasn(tag(21))]
pub struct InitResponse {
    #[rasn(tag(2))]
    pub reference_id: Option<OctetString>,
    #[rasn(tag(3))]
    pub protocol_version: Option<BitString>,
    #[rasn(tag(4))]
    pub options: Option<BitString>,
    #[rasn(tag(5))]
    pub preferred_message_size: Option<Integer>,
    #[rasn(tag(6))]
    pub exceptional_record_size: Option<Integer>,
    #[rasn(tag(12))]
    pub result: bool,
    #[rasn(tag(110))]
    pub implementation_id: Option<Utf8String>,
    #[rasn(tag(111))]
    pub implementation_name: Option<Utf8String>,
    #[rasn(tag(112))]
    pub implementation_version: Option<Utf8String>,
    #[rasn(tag(11))]
    pub user_information_field: Option<OctetString>,
    pub other_info: Option<OctetString>,
}

#[derive(Debug, AsnType, Encode, Decode)]
#[rasn(tag(22))] // Produira 0xB6
pub struct SearchRequest {
    #[rasn(tag(2))]
    pub reference_id: Option<OctetString>,

    #[rasn(tag(13))]
    pub small_set_upper_bound: Integer,

    #[rasn(tag(14))]
    pub large_set_lower_bound: Integer,

    #[rasn(tag(15))]
    pub medium_set_present_number: Integer,

    #[rasn(tag(16))]
    pub replace_indicator: bool,

    #[rasn(tag(17))]
    pub result_set_name: Utf8String,

    #[rasn(tag(18))]
    pub database_names: Vec<DatabaseName>,

    #[rasn(tag(104))]
    pub preferred_record_syntax: Option<ObjectIdentifier>,

    // Voici la syntaxe exacte pour l'EXPLICIT dans rasn :
    #[rasn(tag(explicit(21)))]
    pub query: Query,
}

#[derive(Debug, AsnType, Encode, Decode)]
#[rasn(tag(23))]
pub struct SearchResponse {
    #[rasn(tag(context, 2))]
    pub reference_id: Option<OctetString>,
    #[rasn(tag(context, 23))]
    pub result_count: Integer,
    #[rasn(tag(context, 24))]
    pub number_of_records_returned: Integer,
    #[rasn(tag(context, 25))]
    pub next_result_set_position: Integer,
    #[rasn(tag(context, 22))]
    pub search_status: bool,
    #[rasn(tag(context, 26))]
    pub result_set_status: Option<Integer>,
    #[rasn(tag(context, 27))]
    pub present_status: Option<PresentStatus>,
    pub records: Option<Records>,
    #[rasn(tag(context, 203))]
    pub additional_search_info: Option<OctetString>,
    pub other_info: Option<OctetString>,
}

#[derive(Debug, AsnType, Encode, Decode)]
pub struct PresentRequest {
    #[rasn(tag(context, 2))]
    pub reference_id: Option<OctetString>,
    #[rasn(tag(context, 31))]
    pub result_set_id: Utf8String,
    #[rasn(tag(context, 30))]
    pub result_set_start_point: Integer,
    #[rasn(tag(context, 29))]
    pub number_of_records_requested: Integer,
    #[rasn(tag(context, 212))]
    pub additional_ranges: Option<Vec<Range>>,
    pub record_composition: Option<RecordComposition>,
    #[rasn(tag(context, 104))]
    pub preferred_record_syntax: Option<ObjectIdentifier>,
    #[rasn(tag(context, 204))]
    pub max_segment_count: Option<Integer>,
    #[rasn(tag(context, 206))]
    pub max_record_size: Option<Integer>,
    #[rasn(tag(context, 207))]
    pub max_segment_size: Option<Integer>,
    #[rasn(tag(context, 210))]
    pub other_info: Option<OtherInformation>,
}

#[derive(Debug, AsnType, Encode, Decode)]
pub struct OtherInformation {
    #[rasn(tag(context, 1))]
    pub category: Option<InfoCategory>,

    pub information: OtherInformationChoice,
}

#[derive(AsnType, Encode, Decode, Debug)]
pub struct InfoCategory {
    #[rasn(tag(context, 1))]
    pub category_type_id: Option<ObjectIdentifier>,

    #[rasn(tag(context, 2))]
    pub category_value: Integer,
}

#[derive(AsnType, Encode, Decode, Debug)]
#[rasn(choice)]
pub enum OtherInformationChoice {
    #[rasn(tag(context, 2))]
    CharacterInfo(Utf8String),

    #[rasn(tag(context, 3))]
    BinaryInfo(OctetString),

    #[rasn(tag(context, 4))]
    ExternallyDefinedInfo(External),

    #[rasn(tag(context, 5))]
    Oid(ObjectIdentifier),
}

/// RecordComposition is a CHOICE used in PresentRequest.
/// Since it's optional and has implicit context tags that may conflict with
/// other fields during decoding, we wrap it in explicit tags.
#[derive(Debug, AsnType, Encode, Decode)]
#[rasn(choice)]
pub enum RecordComposition {
    #[rasn(tag(explicit(context, 19)))]
    Simple(ElementSetNames),
    #[rasn(tag(explicit(context, 209)))]
    Complex(CompSpec),
}

#[derive(Debug, AsnType, Encode, Decode)]
#[rasn(choice)]
pub enum ElementSetNames {
    #[rasn(tag(context, 0))]
    GenericElementSetName(Utf8String),
    #[rasn(tag(context, 1))]
    DatabaseSpecific(Vec<DbElementSetName>),
}

#[derive(Debug, AsnType, Encode, Decode)]
pub struct DbElementSetName {
    pub db_name: DatabaseName,
    pub element_set_name: Utf8String,
}

#[derive(Debug, AsnType, Encode, Decode)]
pub struct CompSpec {
    #[rasn(tag(context, 1))]
    pub select_alternative_syntax: bool,
    #[rasn(tag(context, 2))]
    pub generic: Option<Specification>,
    #[rasn(tag(context, 3))]
    pub db_specific: Option<Vec<DbSpecificSpec>>,
    #[rasn(tag(context, 4))]
    pub record_syntax: Option<Vec<ObjectIdentifier>>,
}

#[derive(Debug, AsnType, Encode, Decode)]
pub struct DbSpecificSpec {
    pub db: DatabaseName,
    pub spec: Specification,
}
#[derive(Debug, AsnType, Encode, Decode)]
pub struct Range {
    #[rasn(tag(context, 0))]
    pub starting_position: Integer,
    #[rasn(tag(context, 1))]
    pub number_of_records: Integer,
}

#[derive(Debug, AsnType, Encode, Decode)]
#[rasn(tag(context, 25))]
pub struct PresentResponse {
    #[rasn(tag(context, 2))]
    pub reference_id: Option<OctetString>,
    #[rasn(tag(context, 24))]
    pub number_of_records_returned: Integer, // Obligatoire selon la norme
    #[rasn(tag(context, 25))]
    pub next_result_set_position: Integer, // Obligatoire

    #[rasn(tag(context, 27))]
    pub present_status: Option<PresentStatus>,
    pub records: Option<Records>,
    // Le champ other_info est complexe, souvent taggué 201
    #[rasn(tag(context, 201))]
    pub other_info: Option<OtherInformation>,
}

// ============================================================================
// Delete Result Set (tag 26/27)
// ============================================================================

#[derive(Debug, AsnType, Encode, Decode)]
pub struct DeleteResultSetRequest {
    #[rasn(tag(context, 2))]
    pub reference_id: Option<OctetString>,
    #[rasn(tag(context, 32))]
    pub delete_function: DeleteFunction,
    pub result_set_list: Option<Vec<ResultSetId>>,
    #[rasn(tag(context, 201))]
    pub other_info: Option<OtherInformation>,
}

#[derive(Debug, AsnType, Encode, Decode, Clone, Copy, PartialEq, Eq)]
#[rasn(enumerated)]
pub enum DeleteFunction {
    List = 0,
    All = 1,
}

pub type ResultSetId = Utf8String;

#[derive(Debug, AsnType, Encode, Decode)]
pub struct DeleteResultSetResponse {
    #[rasn(tag(context, 2))]
    pub reference_id: Option<OctetString>,
    #[rasn(tag(context, 33))]
    pub delete_operation_status: DeleteOperationStatus,
    #[rasn(tag(context, 34))]
    pub delete_list_statuses: Option<Vec<ListStatus>>,
    #[rasn(tag(context, 35))]
    pub number_not_deleted: Option<Integer>,
    #[rasn(tag(context, 37))]
    pub bulk_statuses: Option<Vec<ListStatus>>,
    #[rasn(tag(context, 36))]
    pub delete_message: Option<Utf8String>,
    #[rasn(tag(context, 201))]
    pub other_info: Option<OtherInformation>,
}

#[derive(Debug, AsnType, Encode, Decode, Clone, Copy, PartialEq, Eq)]
#[rasn(enumerated)]
pub enum DeleteOperationStatus {
    Success = 0,
    ResultSetDidNotExist = 1,
    PreviouslyDeletedByTarget = 2,
    SystemProblemAtTarget = 3,
    AccessNotAllowed = 4,
    ResourceControlAtOrigin = 5,
    ResourceControlAtTarget = 6,
    BulkDeleteNotSupported = 7,
    NotAllRsltSetsDeletedOnBulkDlte = 8,
    NotAllRequestedResultSetsDeleted = 9,
    ResultSetInUse = 10,
}

#[derive(Debug, AsnType, Encode, Decode)]
pub struct ListStatus {
    pub id: ResultSetId,
    pub status: DeleteOperationStatus,
}

// ============================================================================
// Access Control (tag 28/29)
// ============================================================================

#[derive(Debug, AsnType, Encode, Decode)]
pub struct AccessControlRequest {
    #[rasn(tag(context, 2))]
    pub reference_id: Option<OctetString>,
    pub security_challenge: AccessControlSecurityChallenge,
    #[rasn(tag(context, 201))]
    pub other_info: Option<OtherInformation>,
}

#[derive(Debug, AsnType, Encode, Decode)]
#[rasn(choice)]
pub enum AccessControlSecurityChallenge {
    #[rasn(tag(context, 37))]
    SimpleForm(OctetString),
    #[rasn(tag(context, 0))]
    ExternallyDefined(External),
}

#[derive(Debug, AsnType, Encode, Decode)]
pub struct AccessControlResponse {
    #[rasn(tag(context, 2))]
    pub reference_id: Option<OctetString>,
    pub security_challenge_response: Option<AccessControlSecurityChallengeResponse>,
    #[rasn(tag(context, 223))]
    pub diagnostic: Option<DiagRec>,
    #[rasn(tag(context, 201))]
    pub other_info: Option<OtherInformation>,
}

#[derive(Debug, AsnType, Encode, Decode)]
#[rasn(choice)]
pub enum AccessControlSecurityChallengeResponse {
    #[rasn(tag(context, 38))]
    SimpleForm(OctetString),
    #[rasn(tag(context, 0))]
    ExternallyDefined(External),
}

// ============================================================================
// Resource Control (tag 30/31/32)
// ============================================================================

#[derive(Debug, AsnType, Encode, Decode)]
pub struct ResourceControlRequest {
    #[rasn(tag(context, 2))]
    pub reference_id: Option<OctetString>,
    #[rasn(tag(context, 39))]
    pub suspended_flag: Option<bool>,
    #[rasn(tag(context, 40))]
    pub resource_report: Option<ResourceReport>,
    #[rasn(tag(context, 41))]
    pub partial_results_available: Option<PartialResultsAvailable>,
    #[rasn(tag(context, 46))]
    pub response_required: bool,
    #[rasn(tag(context, 47))]
    pub triggered_request_flag: Option<bool>,
    #[rasn(tag(context, 201))]
    pub other_info: Option<OtherInformation>,
}

#[derive(Debug, AsnType, Encode, Decode, Clone, Copy, PartialEq, Eq)]
#[rasn(enumerated)]
pub enum PartialResultsAvailable {
    Subset = 1,
    Interim = 2,
    None = 3,
}

pub type ResourceReport = External;

#[derive(Debug, AsnType, Encode, Decode)]
pub struct ResourceControlResponse {
    #[rasn(tag(context, 2))]
    pub reference_id: Option<OctetString>,
    #[rasn(tag(context, 44))]
    pub continue_flag: bool,
    #[rasn(tag(context, 45))]
    pub result_set_wanted: Option<bool>,
    #[rasn(tag(context, 201))]
    pub other_info: Option<OtherInformation>,
}

#[derive(Debug, AsnType, Encode, Decode)]
pub struct TriggerResourceControlRequest {
    #[rasn(tag(context, 2))]
    pub reference_id: Option<OctetString>,
    #[rasn(tag(context, 42))]
    pub requested_action: TriggerRequestedAction,
    #[rasn(tag(context, 43))]
    pub preferred_resource_report_format: Option<ObjectIdentifier>,
    #[rasn(tag(context, 48))]
    pub result_set_wanted: Option<bool>,
    #[rasn(tag(context, 201))]
    pub other_info: Option<OtherInformation>,
}

#[derive(Debug, AsnType, Encode, Decode, Clone, Copy, PartialEq, Eq)]
#[rasn(enumerated)]
pub enum TriggerRequestedAction {
    ResourceReport = 1,
    ResourceControl = 2,
    Cancel = 3,
}

// ============================================================================
// Resource Report (tag 33/34)
// ============================================================================

#[derive(Debug, AsnType, Encode, Decode)]
pub struct ResourceReportRequest {
    #[rasn(tag(context, 2))]
    pub reference_id: Option<OctetString>,
    #[rasn(tag(context, 49))]
    pub op_id: Option<ReferenceId>,
    #[rasn(tag(context, 43))]
    pub preferred_resource_report_format: Option<ObjectIdentifier>,
    #[rasn(tag(context, 201))]
    pub other_info: Option<OtherInformation>,
}

pub type ReferenceId = OctetString;

#[derive(Debug, AsnType, Encode, Decode)]
pub struct ResourceReportResponse {
    #[rasn(tag(context, 2))]
    pub reference_id: Option<OctetString>,
    #[rasn(tag(context, 50))]
    pub resource_report_status: ResourceReportStatus,
    #[rasn(tag(context, 40))]
    pub resource_report: Option<ResourceReport>,
    #[rasn(tag(context, 201))]
    pub other_info: Option<OtherInformation>,
}

#[derive(Debug, AsnType, Encode, Decode, Clone, Copy, PartialEq, Eq)]
#[rasn(enumerated)]
pub enum ResourceReportStatus {
    Success = 0,
    Partial = 1,
    FailureNoReport = 2,
    FailureNoReportNoEstimate = 3,
    FailureNoReportCannotDetermine = 4,
    FailureNoReportDueToCondition = 5,
    FailureReportNotSupported = 6,
    FailureReportFormatNotSupported = 7,
}

// ============================================================================
// Scan (tag 35/36)
// ============================================================================

#[derive(Debug, AsnType, Encode, Decode)]
pub struct ScanRequest {
    #[rasn(tag(context, 2))]
    pub reference_id: Option<OctetString>,
    #[rasn(tag(context, 3))]
    pub database_names: Vec<DatabaseName>,
    #[rasn(tag(context, 4))]
    pub attribute_set: Option<ObjectIdentifier>,
    pub terms_list_and_start_point: AttributesPlusTerm,
    #[rasn(tag(context, 5))]
    pub step_size: Option<Integer>,
    #[rasn(tag(context, 6))]
    pub number_of_terms_requested: Integer,
    #[rasn(tag(context, 7))]
    pub preferred_position_in_response: Option<Integer>,
    #[rasn(tag(context, 201))]
    pub other_info: Option<OtherInformation>,
}

#[derive(Debug, AsnType, Encode, Decode)]
pub struct ScanResponse {
    #[rasn(tag(context, 2))]
    pub reference_id: Option<OctetString>,
    #[rasn(tag(context, 3))]
    pub step_size: Option<Integer>,
    #[rasn(tag(context, 4))]
    pub scan_status: ScanStatus,
    #[rasn(tag(context, 5))]
    pub number_of_entries_returned: Integer,
    #[rasn(tag(context, 6))]
    pub position_of_term: Option<Integer>,
    #[rasn(tag(context, 7))]
    pub entries: Option<ListEntries>,
    #[rasn(tag(context, 8))]
    pub attribute_set: Option<ObjectIdentifier>,
    #[rasn(tag(context, 201))]
    pub other_info: Option<OtherInformation>,
}

#[derive(Debug, AsnType, Encode, Decode, Clone, Copy, PartialEq, Eq)]
#[rasn(enumerated)]
pub enum ScanStatus {
    Success = 0,
    PartialBeginning = 1,
    PartialEnd = 2,
    PartialBoth = 3,
    PartialEmpty = 4,
    PartialEstimate = 5,
    Failure = 6,
}

#[derive(Debug, AsnType, Encode, Decode)]
pub struct ListEntries {
    pub entries: Option<Vec<Entry>>,
    #[rasn(tag(context, 2))]
    pub nonsurrogate_diagnostics: Option<Vec<DiagRec>>,
}

#[derive(Debug, AsnType, Encode, Decode)]
#[rasn(choice)]
pub enum Entry {
    #[rasn(tag(context, 1))]
    TermInfo(TermInfo),
    #[rasn(tag(context, 2))]
    SurrogateDiagnostic(DiagRec),
}

#[derive(Debug, AsnType, Encode, Decode)]
pub struct TermInfo {
    pub term: Term,
    #[rasn(tag(context, 1))]
    pub display_term: Option<Utf8String>,
    pub suggested_attributes: Option<AttributeList>,
    #[rasn(tag(context, 2))]
    pub alternative_term: Option<Vec<AttributesPlusTerm>>,
    #[rasn(tag(context, 3))]
    pub global_occurrences: Option<Integer>,
    #[rasn(tag(context, 4))]
    pub by_attributes: Option<OccurrenceByAttributes>,
    #[rasn(tag(context, 201))]
    pub other_term_info: Option<OtherInformation>,
}

#[derive(Debug, AsnType, Encode, Decode)]
pub struct OccurrenceByAttributes {
    pub occurrences: Vec<OccurrenceByAttributesElem>,
}

#[derive(Debug, AsnType, Encode, Decode)]
pub struct OccurrenceByAttributesElem {
    pub attributes: AttributeList,
    pub occurrences: Option<OccurrencesValue>,
}

#[derive(Debug, AsnType, Encode, Decode)]
#[rasn(choice)]
pub enum OccurrencesValue {
    #[rasn(tag(context, 2))]
    Global(Integer),
    #[rasn(tag(context, 3))]
    ByDatabase(Vec<OccurrenceByDatabase>),
}

#[derive(Debug, AsnType, Encode, Decode)]
pub struct OccurrenceByDatabase {
    pub db: DatabaseName,
    pub num: Option<Integer>,
    #[rasn(tag(context, 201))]
    pub other_db_info: Option<OtherInformation>,
}

// ============================================================================
// Sort (tag 43/44)
// ============================================================================

#[derive(Debug, AsnType, Encode, Decode)]
pub struct SortRequest {
    #[rasn(tag(context, 2))]
    pub reference_id: Option<OctetString>,
    #[rasn(tag(context, 3))]
    pub input_result_set_names: Vec<Utf8String>,
    #[rasn(tag(context, 4))]
    pub sorted_result_set_name: Utf8String,
    #[rasn(tag(context, 5))]
    pub sort_sequence: Vec<SortKeySpec>,
    #[rasn(tag(context, 201))]
    pub other_info: Option<OtherInformation>,
}

#[derive(Debug, AsnType, Encode, Decode)]
pub struct SortKeySpec {
    pub sort_element: SortElement,
    #[rasn(tag(context, 1))]
    pub sort_relation: SortRelation,
    #[rasn(tag(context, 2))]
    pub case_sensitivity: CaseSensitivity,
    #[rasn(tag(context, 3))]
    pub missing_value_action: Option<MissingValueAction>,
}

#[derive(Debug, AsnType, Encode, Decode)]
#[rasn(choice)]
pub enum SortElement {
    #[rasn(tag(context, 1))]
    Generic(SortKey),
    #[rasn(tag(context, 2))]
    DataBaseSpecific(Vec<SortDbSpecific>),
}

#[derive(Debug, AsnType, Encode, Decode)]
pub struct SortDbSpecific {
    pub database_name: DatabaseName,
    pub db_sort: SortKey,
}

#[derive(Debug, AsnType, Encode, Decode)]
#[rasn(choice)]
pub enum SortKey {
    #[rasn(tag(context, 0))]
    SortField(Utf8String),
    #[rasn(tag(context, 1))]
    ElementSpec(Specification),
    #[rasn(tag(context, 2))]
    SortAttributes(SortAttributes),
}

#[derive(Debug, AsnType, Encode, Decode)]
pub struct SortAttributes {
    pub id: ObjectIdentifier,
    pub list: AttributeList,
}

#[derive(Debug, AsnType, Encode, Decode)]
pub struct Specification {
    #[rasn(tag(context, 1))]
    pub schema: Option<ObjectIdentifier>,
    #[rasn(tag(context, 2))]
    pub element_spec: Option<ElementSpec>,
}

#[derive(Debug, AsnType, Encode, Decode)]
#[rasn(choice)]
pub enum ElementSpec {
    #[rasn(tag(context, 1))]
    ElementSetName(Utf8String),
    #[rasn(tag(context, 2))]
    ExternalEspec(External),
}

#[derive(Debug, AsnType, Encode, Decode, Clone, Copy, PartialEq, Eq)]
#[rasn(enumerated)]
pub enum SortRelation {
    Ascending = 0,
    Descending = 1,
    AscendingByFrequency = 3,
    DescendingByFrequency = 4,
}

#[derive(Debug, AsnType, Encode, Decode, Clone, Copy, PartialEq, Eq)]
#[rasn(enumerated)]
pub enum CaseSensitivity {
    CaseSensitive = 0,
    CaseInsensitive = 1,
}

#[derive(Debug, AsnType, Encode, Decode)]
#[rasn(choice)]
pub enum MissingValueAction {
    #[rasn(tag(context, 1))]
    Abort(()),
    #[rasn(tag(context, 2))]
    Null(()),
    #[rasn(tag(context, 3))]
    MissingValueData(OctetString),
}

#[derive(Debug, AsnType, Encode, Decode)]
pub struct SortResponse {
    #[rasn(tag(context, 2))]
    pub reference_id: Option<OctetString>,
    #[rasn(tag(context, 3))]
    pub sort_status: SortStatus,
    #[rasn(tag(context, 4))]
    pub result_set_status: Option<SortResultSetStatus>,
    #[rasn(tag(context, 5))]
    pub diagnostics: Option<Vec<DiagRec>>,
    #[rasn(tag(context, 6))]
    pub result_count: Option<Integer>,
    #[rasn(tag(context, 201))]
    pub other_info: Option<OtherInformation>,
}

#[derive(Debug, AsnType, Encode, Decode, Clone, Copy, PartialEq, Eq)]
#[rasn(enumerated)]
pub enum SortStatus {
    Success = 0,
    PartialResultsAvailable = 1,
    Failure = 2,
}

#[derive(Debug, AsnType, Encode, Decode, Clone, Copy, PartialEq, Eq)]
#[rasn(enumerated)]
pub enum SortResultSetStatus {
    Empty = 1,
    Interim = 2,
    Unchanged = 3,
    None = 4,
}

// ============================================================================
// Segment (tag 45)
// ============================================================================

#[derive(Debug, AsnType, Encode, Decode)]
pub struct Segment {
    #[rasn(tag(context, 2))]
    pub reference_id: Option<OctetString>,
    #[rasn(tag(context, 24))]
    pub number_of_records_returned: Integer,
    #[rasn(tag(context, 0))]
    pub segment_records: Vec<NamePlusRecord>,
    #[rasn(tag(context, 201))]
    pub other_info: Option<OtherInformation>,
}

// ============================================================================
// Extended Services (tag 46/47)
// ============================================================================

#[derive(Debug, AsnType, Encode, Decode)]
pub struct ExtendedServicesRequest {
    #[rasn(tag(context, 2))]
    pub reference_id: Option<OctetString>,
    #[rasn(tag(context, 3))]
    pub function: ExtendedServicesFunction,
    #[rasn(tag(context, 4))]
    pub package_type: ObjectIdentifier,
    #[rasn(tag(context, 5))]
    pub package_name: Option<Utf8String>,
    #[rasn(tag(context, 6))]
    pub user_id: Option<Utf8String>,
    #[rasn(tag(context, 7))]
    pub retention_time: Option<IntUnit>,
    #[rasn(tag(context, 8))]
    pub permissions: Option<Permissions>,
    #[rasn(tag(context, 9))]
    pub description: Option<Utf8String>,
    #[rasn(tag(context, 10))]
    pub task_specific_parameters: Option<External>,
    #[rasn(tag(context, 11))]
    pub wait_action: WaitAction,
    #[rasn(tag(context, 103))]
    pub elements: Option<ElementSetName>,
    #[rasn(tag(context, 201))]
    pub other_info: Option<OtherInformation>,
}

#[derive(Debug, AsnType, Encode, Decode, Clone, Copy, PartialEq, Eq)]
#[rasn(enumerated)]
pub enum ExtendedServicesFunction {
    Create = 1,
    Delete = 2,
    Modify = 3,
}

#[derive(Debug, AsnType, Encode, Decode, Clone, Copy, PartialEq, Eq)]
#[rasn(enumerated)]
pub enum WaitAction {
    Wait = 1,
    WaitIfPossible = 2,
    DontWait = 3,
    DontReturnPackage = 4,
}

pub type ElementSetName = Utf8String;

#[derive(Debug, AsnType, Encode, Decode)]
pub struct Permissions {
    pub permissions: Vec<PermissionsElem>,
}

#[derive(Debug, AsnType, Encode, Decode)]
pub struct PermissionsElem {
    #[rasn(tag(context, 1))]
    pub user_id: Utf8String,
    #[rasn(tag(context, 2))]
    pub allowed_functions: Vec<AllowedFunction>,
}

#[derive(Debug, AsnType, Encode, Decode, Clone, Copy, PartialEq, Eq)]
#[rasn(enumerated)]
pub enum AllowedFunction {
    Delete = 1,
    ModifyContents = 2,
    ModifyPermissions = 3,
    Present = 4,
    Invoke = 5,
}

#[derive(Debug, AsnType, Encode, Decode)]
pub struct ExtendedServicesResponse {
    #[rasn(tag(context, 2))]
    pub reference_id: Option<OctetString>,
    #[rasn(tag(context, 3))]
    pub operation_status: ExtendedServicesStatus,
    #[rasn(tag(context, 4))]
    pub diagnostics: Option<Vec<DiagRec>>,
    #[rasn(tag(context, 5))]
    pub task_package: Option<External>,
    #[rasn(tag(context, 201))]
    pub other_info: Option<OtherInformation>,
}

#[derive(Debug, AsnType, Encode, Decode, Clone, Copy, PartialEq, Eq)]
#[rasn(enumerated)]
pub enum ExtendedServicesStatus {
    Done = 1,
    Accepted = 2,
    Failure = 3,
}

// ============================================================================
// Close (tag 48)
// ============================================================================

#[derive(Debug, AsnType, Encode, Decode)]
pub struct Close {
    #[rasn(tag(context, 2))]
    pub reference_id: Option<OctetString>,
    #[rasn(tag(context, 211))]
    pub close_reason: CloseReason,
    #[rasn(tag(context, 3))]
    pub diagnostic_information: Option<Utf8String>,
    #[rasn(tag(context, 4))]
    pub resource_report_format: Option<ObjectIdentifier>,
    #[rasn(tag(context, 5))]
    pub resource_report: Option<ResourceReport>,
    #[rasn(tag(context, 201))]
    pub other_info: Option<OtherInformation>,
}

#[derive(Debug, AsnType, Encode, Decode, Clone, Copy, PartialEq, Eq)]
#[rasn(enumerated)]
pub enum CloseReason {
    Finished = 0,
    Shutdown = 1,
    SystemProblem = 2,
    CostLimit = 3,
    Resources = 4,
    SecurityViolation = 5,
    ProtocolError = 6,
    LackOfActivity = 7,
    PeerAbort = 8,
    Unspecified = 9,
}

// ============================================================================
// Duplicate Detection (tag 49/50)
// ============================================================================

#[derive(Debug, AsnType, Encode, Decode)]
pub struct DuplicateDetectionRequest {
    #[rasn(tag(context, 2))]
    pub reference_id: Option<OctetString>,
    #[rasn(tag(context, 3))]
    pub input_result_set_ids: Vec<Utf8String>,
    #[rasn(tag(context, 4))]
    pub output_result_set_name: Utf8String,
    #[rasn(tag(context, 5))]
    pub applicable_portion: Option<ApplicablePortion>,
    #[rasn(tag(context, 6))]
    pub duplicate_detection_criteria: Option<Vec<DuplicateDetectionCriterion>>,
    #[rasn(tag(context, 7))]
    pub clustering: Option<bool>,
    #[rasn(tag(context, 8))]
    pub retention_criteria: Option<Vec<RetentionCriterion>>,
    #[rasn(tag(context, 9))]
    pub sorting_criteria: Option<Vec<SortCriterion>>,
    #[rasn(tag(context, 201))]
    pub other_info: Option<OtherInformation>,
}

#[derive(Debug, AsnType, Encode, Decode)]
#[rasn(choice)]
pub enum ApplicablePortion {
    #[rasn(tag(context, 2))]
    Full(()),
    #[rasn(tag(context, 3))]
    Fields(Vec<Utf8String>),
}

#[derive(Debug, AsnType, Encode, Decode)]
#[rasn(choice)]
pub enum DuplicateDetectionCriterion {
    #[rasn(tag(context, 1))]
    LevelOfMatch(Integer),
    #[rasn(tag(context, 2))]
    CaseSensitive(bool),
    #[rasn(tag(context, 3))]
    PunctuationSensitive(bool),
    #[rasn(tag(context, 4))]
    RegularExpression(External),
    #[rasn(tag(context, 5))]
    RsDuplicates(External),
}

#[derive(Debug, AsnType, Encode, Decode)]
#[rasn(choice)]
pub enum RetentionCriterion {
    #[rasn(tag(context, 1))]
    NumberOfEntries(Integer),
    #[rasn(tag(context, 2))]
    PercentOfEntries(Integer),
    #[rasn(tag(context, 3))]
    DuplicatesOnly(()),
    #[rasn(tag(context, 4))]
    DiscardRsDuplicates(()),
}

#[derive(Debug, AsnType, Encode, Decode)]
#[rasn(choice)]
pub enum SortCriterion {
    #[rasn(tag(context, 1))]
    MostComprehensive(()),
    #[rasn(tag(context, 2))]
    LeastComprehensive(()),
    #[rasn(tag(context, 3))]
    MostRecent(()),
    #[rasn(tag(context, 4))]
    Oldest(()),
    #[rasn(tag(context, 5))]
    LeastCostPerRecord(()),
    #[rasn(tag(context, 6))]
    PreferredDatabases(Vec<DatabaseName>),
}

#[derive(Debug, AsnType, Encode, Decode)]
pub struct DuplicateDetectionResponse {
    #[rasn(tag(context, 2))]
    pub reference_id: Option<OctetString>,
    #[rasn(tag(context, 3))]
    pub status: DuplicateDetectionStatus,
    #[rasn(tag(context, 4))]
    pub result_set_count: Option<Integer>,
    #[rasn(tag(context, 5))]
    pub diagnostics: Option<Vec<DiagRec>>,
    #[rasn(tag(context, 201))]
    pub other_info: Option<OtherInformation>,
}

#[derive(Debug, AsnType, Encode, Decode, Clone, Copy, PartialEq, Eq)]
#[rasn(enumerated)]
pub enum DuplicateDetectionStatus {
    Success = 0,
    PartialSomeInputNotProcessed = 1,
    PartialSomeCriteriaNotApplied = 2,
    PartialBothProblems = 3,
    Failure = 4,
}

#[derive(Debug, AsnType, Encode, Decode)]
#[rasn(choice)]
pub enum Records {
    #[rasn(tag(context, 28))]
    ResponseRecords(Vec<NamePlusRecord>),
    #[rasn(tag(context, 130))]
    NonSurrogateDiagnostic(DefaultDiagFormat),
    #[rasn(tag(context, 205))]
    MultipeNonSurDiagnostic(Vec<DiagRec>),
}

#[derive(AsnType, Encode, Decode, Debug)]
#[rasn(choice)]
pub enum DiagRec {
    DefaultFormat(DefaultDiagFormat),
    ExternallDefined(External),
}

#[derive(AsnType, Encode, Decode, Debug)]
pub struct DefaultDiagFormat {
    pub diagnostic_set_id: ObjectIdentifier,
    pub condition: Integer,
    pub addinfo: AddInfo,
}

#[derive(AsnType, Encode, Decode, Debug)]
#[rasn(choice)]
pub enum AddInfo {
    V2Addinfo(VisibleString),
    V3Addinfo(Utf8String),
}

#[derive(Debug, AsnType, Encode, Decode)]
pub struct NamePlusRecord {
    #[rasn(tag(context, 0))]
    pub name: Option<VisibleString>,

    #[rasn(tag(explicit(context, 1)))] // Premier 161
    pub record: Record,
}

#[derive(Debug, AsnType, Encode, Decode)]
#[rasn(choice)]
pub enum Record {
    #[rasn(tag(explicit(context, 1)))]
    RetrievalRecord(External),
    #[rasn(tag(context, 2))]
    SurrogateDiagnostic(DiagRec),
    #[rasn(tag(context, 3))]
    StartingFragment(FragmentSyntax),
    #[rasn(tag(context, 4))]
    IntermediateFragment(FragmentSyntax),
    #[rasn(tag(context, 5))]
    FinalFragment(FragmentSyntax),
}

#[derive(Debug, AsnType, Encode, Decode)]
#[rasn(choice)]
pub enum FragmentSyntax {
    ExternallyTagged(External),
    NotExternallyTagged(OctetString),
}

#[derive(Debug, AsnType, Encode, Decode)]
#[rasn(tag(universal, 8))]
pub struct External {
    pub direct_reference: Option<ObjectIdentifier>,
    pub indirect_reference: Option<Integer>,
    pub data_value_descriptor: Option<Utf8String>,
    pub encoding: ExternalEncoding,
}

#[derive(Debug, AsnType, Encode, Decode)]
#[rasn(choice)]
pub enum ExternalEncoding {
    #[rasn(tag(context, 0))]
    SingleASN1Type(Any),
    #[rasn(tag(context, 1))]
    OctetAligned(OctetString),
    #[rasn(tag(context, 2))]
    Arbitrary(BitString),
}

#[derive(Debug, AsnType, Encode, Decode, Clone, Copy, PartialEq, Eq)]
#[rasn(enumerated)]
pub enum PresentStatus {
    Success = 0,
    Partial1 = 1,
    Partial2 = 2,
    Partial3 = 3,
    Partial4 = 4,
    Failure = 5,
}

#[derive(Debug, AsnType, Encode, Decode)]
#[rasn(choice)]
pub enum IdAuthentication {
    #[rasn(tag(universal, 26))]
    Open(VisibleString),
    #[rasn(tag(context, 1))]
    IdPass(IdPass),
}

#[derive(Debug, AsnType, Encode, Decode)]
pub struct IdPass {
    #[rasn(tag(context, 0))]
    pub id: Utf8String,
    #[rasn(tag(context, 1))]
    pub password: Utf8String,
}

#[derive(Debug, AsnType, Encode, Decode)]
#[rasn(choice)]
pub enum Query {
    // [1] EXPLICIT RPNQuery (Type-1 standard souvent utilisé)
    #[rasn(tag(context, 1))]
    Type1(RpnQuery),

    #[rasn(tag(context, 2))]
    Type2(RpnQuery),

    // type-100 [100] OCTET STRING
    #[rasn(tag(context, 100))]
    Type100(OctetString),

    // type-101 [101] IMPLICIT RPNQuery
    // Note: rasn utilise l'implicite par défaut, donc tag(context, 101) suffit
    #[rasn(tag(context, 101))]
    Type101(RpnQuery),

    // type-102 [102] OCTET STRING
    #[rasn(tag(context, 102))]
    Type102(OctetString),
}

#[derive(Debug, AsnType, Encode, Decode)]
pub struct RpnQuery {
    pub attribute_set: ObjectIdentifier,
    pub rpn: RpnStructure,
}

#[derive(Debug, AsnType, Encode, Decode)]
#[rasn(choice)]
pub enum RpnStructure {
    #[rasn(tag(context, 0))]
    Op(Operand),
    #[rasn(tag(context, 1))]
    RpnRpnOperator(RpnRpnOperator),
}

#[derive(Debug, AsnType, Encode, Decode)]
pub struct RpnRpnOperator {
    pub rpn1: Box<RpnStructure>, // Box est nécessaire pour la récursion
    pub rpn2: Box<RpnStructure>,
    pub op: Operator,
}

#[derive(Debug, AsnType, Encode, Decode)]
#[rasn(choice)]
pub enum Operator {
    #[rasn(tag(context, 0))]
    And(()),
    #[rasn(tag(context, 1))]
    Or(()),
    #[rasn(tag(context, 2))]
    AndNot(()),
}

#[derive(Debug, AsnType, Encode, Decode)]
#[rasn(choice)]
pub enum Operand {
    #[rasn(tag(context, 102))]
    AttributesPlusTerm(AttributesPlusTerm),
    #[rasn(tag(context, 33))]
    ResultSet(Utf8String),
    #[rasn(tag(context, 214))]
    ResultAttr(ResultSetPlusAttributes),
}

#[derive(Debug, AsnType, Encode, Decode)]
pub struct ResultSetPlusAttributes {
    pub result_set: Utf8String,
    pub attributes: AttributeList,
}

#[derive(Debug, AsnType, Encode, Decode)]
pub struct AttributesPlusTerm {
    #[rasn(tag(context, 44))]
    pub attributes: Vec<AttributeElement>,
    pub term: Term,
}

#[derive(Debug, AsnType, Encode, Decode)]
pub struct AttributeList {
    pub elements: Vec<AttributeElement>,
}

#[derive(Debug, AsnType, Encode, Decode)]
pub struct AttributeElement {
    #[rasn(tag(context, 1))]
    pub attribute_set: Option<ObjectIdentifier>,
    #[rasn(tag(context, 120))]
    pub attribute_type: Integer,
    pub attribute_value: AttributeValue,
}

#[derive(Debug, AsnType, Encode, Decode)]
#[rasn(choice)]
pub enum AttributeValue {
    #[rasn(tag(context, 121))]
    Numeric(Integer),
}

#[derive(Debug, AsnType, Encode, Decode)]
#[rasn(choice)]
pub enum Term {
    #[rasn(tag(45))]
    General(OctetString),

    #[rasn(tag(context, 215))]
    Numeric(Integer),
    #[rasn(tag(context, 216))]
    CharacterString(Utf8String),
    #[rasn(tag(context, 217))]
    Oid(ObjectIdentifier),
    #[rasn(tag(context, 218))]
    DateTime(GeneralizedTime),
    #[rasn(tag(context, 219))]
    External(External),
    #[rasn(tag(context, 220))]
    IntegerAndUnit(IntUnit),
    #[rasn(tag(context, 221))]
    Null(()),
}

#[derive(Debug, AsnType, Encode, Decode)]
#[rasn(choice)]
pub enum IntUnit {
    #[rasn(tag(context, 1))]
    Value(Integer),
    #[rasn(tag(context, 2))]
    Unit(Unit),
}

#[derive(Debug, AsnType, Encode, Decode)]

pub struct Unit {
    #[rasn(tag(context, 1))]
    pub unit_system: Option<Utf8String>,
    #[rasn(tag(context, 2))]
    pub unit_type: Option<StringOrNumeric>,
    #[rasn(tag(context, 3))]
    pub unit: Option<StringOrNumeric>,
}

#[derive(Debug, AsnType, Encode, Decode)]
#[rasn(choice)]
pub enum StringOrNumeric {
    #[rasn(tag(context, 1))]
    String(Utf8String),
    #[rasn(tag(context, 2))]
    Numeric(Integer),
}

/// BIB-1 attribute set OID (1.2.840.10003.3.1).
pub fn bib1_attribute_set() -> Result<ObjectIdentifier> {
    ObjectIdentifier::new(vec![1, 2, 840, 10003, 3, 1]).ok_or_else(|| Error::InvalidOid("failed to construct BIB-1 attribute set OID".into()))
}

/// USMARC record syntax OID (1.2.840.10003.5.10).
pub fn record_syntax_usmarc() -> Result<ObjectIdentifier> {
    ObjectIdentifier::new(vec![1, 2, 840, 10003, 5, 10]).ok_or_else(|| Error::InvalidOid("failed to construct USMARC record syntax OID".into()))
}

/// Builds a minimal InitRequest (Z39.50 Init APDU).
///
/// This function is intentionally small and strict:
/// - It only emits standards-compliant fields
/// - It validates that strings fit the ASN.1 types we encode (e.g. VisibleString)
pub fn make_init_request(auth: Option<&Credentials>) -> Result<InitRequest> {
    // Protocol version bitstring: bits 0..=2 are set to support v1/v2/v3.
    let mut protocol_version = BitString::with_capacity(16);
    protocol_version.push(true); // bit 0: v1
    protocol_version.push(true); // bit 1: v2
    protocol_version.push(true); // bit 2: v3
    for _ in 3..16 {
        protocol_version.push(false);
    }

    // Options bitstring: bits 0..=1 enable Search and Present.
    let mut options = BitString::with_capacity(32);
    options.push(true); // bit 0: search
    options.push(true); // bit 1: present
    for _ in 2..32 {
        options.push(false);
    }

    let id_authentication = match auth {
        None => None,
        Some(c) => {
            // Common "Open" authentication convention: `username/password`.
            let combined = format!("{}/{}", c.username, c.password);
            let vs = VisibleString::from_iso646_bytes(combined.as_bytes()).map_err(|e| Error::InvalidVisibleString(e.to_string()))?;
            Some(IdAuthentication::Open(vs))
        }
    };

    Ok(InitRequest {
        reference_id: None,
        protocol_version,
        options,
        preferred_message_size: 0x04000000i64.into(),
        exceptional_record_size: 0x04000000i64.into(),
        id_authentication,
        implementation_id: Some(Utf8String::from("81")),
        implementation_name: Some(Utf8String::from("YAZ")),
        implementation_version: Some(Utf8String::from("5.34.4 b42e25e840666ea3422c3bd5cb566b07f78a99cd")),
        user_information_field: None,
        other_info: None,
    })
}

/// Builds a Type-1 query (RPN query with BIB-1 attributes).
pub fn make_type1_query(attribute_type: i64, term: &str) -> Result<Query> {
    let attr = AttributeElement {
        attribute_set: Some(bib1_attribute_set()?),
        attribute_type: attribute_type.into(),
        attribute_value: AttributeValue::Numeric(1.into()),
    };

    let rpn = RpnQuery {
        attribute_set: bib1_attribute_set()?,
        rpn: RpnStructure::Op(Operand::AttributesPlusTerm(AttributesPlusTerm {
            attributes: vec![attr],

            term: Term::General(OctetString::from(term.as_bytes().to_vec())),
        })),
    };

    Ok(Query::Type1(rpn))
}

/// Builds a basic SearchRequest using the provided database names and result set name.
pub fn make_search_request(databases: &[String], result_set: &str, query: Query) -> Result<SearchRequest> {
    let database_names = databases
        .iter()
        .cloned()
        .map(|s| {
            let vs = VisibleString::from_iso646_bytes(s.as_bytes()).map_err(|e| Error::InvalidVisibleString(e.to_string()))?;
            Ok(DatabaseName::General(vs))
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(SearchRequest {
        reference_id: None,
        database_names,
        small_set_upper_bound: 0.into(),
        large_set_lower_bound: 1.into(),
        medium_set_present_number: 0.into(),
        replace_indicator: true,
        result_set_name: Utf8String::from(result_set),
        preferred_record_syntax: None,
        query,
    })
}

/// Builds a PresentRequest for the current result set.
pub fn make_present_request(result_set: &str, start: i64, count: i64) -> Result<PresentRequest> {
    Ok(PresentRequest {
        reference_id: None,
        result_set_id: Utf8String::from(result_set),
        result_set_start_point: start.into(),
        number_of_records_requested: count.into(),
        preferred_record_syntax: Some(record_syntax_usmarc()?),
        other_info: None,
        additional_ranges: None,
        record_composition: None,
        max_segment_count: None,
        max_record_size: None,
        max_segment_size: None,
    })
}

/// Builds a DeleteResultSetRequest to delete specific result sets.
pub fn make_delete_result_set_request(result_sets: &[&str]) -> DeleteResultSetRequest {
    DeleteResultSetRequest {
        reference_id: None,
        delete_function: DeleteFunction::List,
        result_set_list: Some(result_sets.iter().map(|s| Utf8String::from(*s)).collect()),
        other_info: None,
    }
}

/// Builds a DeleteResultSetRequest to delete all result sets.
pub fn make_delete_all_result_sets_request() -> DeleteResultSetRequest {
    DeleteResultSetRequest {
        reference_id: None,
        delete_function: DeleteFunction::All,
        result_set_list: None,
        other_info: None,
    }
}

/// Builds a ScanRequest to browse an index.
pub fn make_scan_request(
    databases: &[String],
    term: &str,
    attribute_type: i64,
    step_size: Option<i64>,
    number_of_terms: i64,
    preferred_position: Option<i64>,
) -> Result<ScanRequest> {
    let database_names = databases
        .iter()
        .cloned()
        .map(|s| {
            let vs = VisibleString::from_iso646_bytes(s.as_bytes())
                .map_err(|e| Error::InvalidVisibleString(e.to_string()))?;
            Ok(DatabaseName::General(vs))
        })
        .collect::<Result<Vec<_>>>()?;

    let attr = AttributeElement {
        attribute_set: Some(bib1_attribute_set()?),
        attribute_type: attribute_type.into(),
        attribute_value: AttributeValue::Numeric(1.into()),
    };

    Ok(ScanRequest {
        reference_id: None,
        database_names,
        attribute_set: Some(bib1_attribute_set()?),
        terms_list_and_start_point: AttributesPlusTerm {
            attributes: vec![attr],
            term: Term::General(OctetString::from(term.as_bytes().to_vec())),
        },
        step_size: step_size.map(|s| s.into()),
        number_of_terms_requested: number_of_terms.into(),
        preferred_position_in_response: preferred_position.map(|p| p.into()),
        other_info: None,
    })
}

/// Builds a SortRequest to sort result sets.
pub fn make_sort_request(
    input_result_sets: &[&str],
    output_result_set: &str,
    sort_keys: Vec<SortKeySpec>,
) -> SortRequest {
    SortRequest {
        reference_id: None,
        input_result_set_names: input_result_sets.iter().map(|s| Utf8String::from(*s)).collect(),
        sorted_result_set_name: Utf8String::from(output_result_set),
        sort_sequence: sort_keys,
        other_info: None,
    }
}

/// Builds a simple SortKeySpec for sorting by a field name.
pub fn make_sort_key_by_field(
    field_name: &str,
    ascending: bool,
    case_sensitive: bool,
) -> SortKeySpec {
    SortKeySpec {
        sort_element: SortElement::Generic(SortKey::SortField(Utf8String::from(field_name))),
        sort_relation: if ascending { SortRelation::Ascending } else { SortRelation::Descending },
        case_sensitivity: if case_sensitive { CaseSensitivity::CaseSensitive } else { CaseSensitivity::CaseInsensitive },
        missing_value_action: None,
    }
}

/// Builds a Close request.
pub fn make_close_request(reason: CloseReason, diagnostic_info: Option<&str>) -> Close {
    Close {
        reference_id: None,
        close_reason: reason,
        diagnostic_information: diagnostic_info.map(|s| Utf8String::from(s)),
        resource_report_format: None,
        resource_report: None,
        other_info: None,
    }
}

/// Builds an ExtendedServicesRequest.
pub fn make_extended_services_request(
    function: ExtendedServicesFunction,
    package_type: ObjectIdentifier,
    package_name: Option<&str>,
    task_specific_parameters: Option<External>,
    wait_action: WaitAction,
) -> ExtendedServicesRequest {
    ExtendedServicesRequest {
        reference_id: None,
        function,
        package_type,
        package_name: package_name.map(|s| Utf8String::from(s)),
        user_id: None,
        retention_time: None,
        permissions: None,
        description: None,
        task_specific_parameters,
        wait_action,
        elements: None,
        other_info: None,
    }
}

/// Builds a DuplicateDetectionRequest.
pub fn make_duplicate_detection_request(
    input_result_sets: &[&str],
    output_result_set: &str,
    clustering: bool,
) -> DuplicateDetectionRequest {
    DuplicateDetectionRequest {
        reference_id: None,
        input_result_set_ids: input_result_sets.iter().map(|s| Utf8String::from(*s)).collect(),
        output_result_set_name: Utf8String::from(output_result_set),
        applicable_portion: None,
        duplicate_detection_criteria: None,
        clustering: Some(clustering),
        retention_criteria: None,
        sorting_criteria: None,
        other_info: None,
    }
}

/// Builds a ResourceControlResponse.
pub fn make_resource_control_response(continue_flag: bool, result_set_wanted: Option<bool>) -> ResourceControlResponse {
    ResourceControlResponse {
        reference_id: None,
        continue_flag,
        result_set_wanted,
        other_info: None,
    }
}

/// Builds an AccessControlResponse with simple form.
pub fn make_access_control_response(response: &[u8]) -> AccessControlResponse {
    AccessControlResponse {
        reference_id: None,
        security_challenge_response: Some(AccessControlSecurityChallengeResponse::SimpleForm(
            OctetString::from(response.to_vec()),
        )),
        diagnostic: None,
        other_info: None,
    }
}

/// Builds a TriggerResourceControlRequest.
pub fn make_trigger_resource_control_request(
    action: TriggerRequestedAction,
    result_set_wanted: Option<bool>,
) -> TriggerResourceControlRequest {
    TriggerResourceControlRequest {
        reference_id: None,
        requested_action: action,
        preferred_resource_report_format: None,
        result_set_wanted,
        other_info: None,
    }
}

/// Builds a ResourceReportRequest.
pub fn make_resource_report_request(
    op_id: Option<&[u8]>,
    preferred_format: Option<ObjectIdentifier>,
) -> ResourceReportRequest {
    ResourceReportRequest {
        reference_id: None,
        op_id: op_id.map(|b| OctetString::from(b.to_vec())),
        preferred_resource_report_format: preferred_format,
        other_info: None,
    }
}

/// Extracts raw MARC records from a PresentResponse.
///
/// Returns an error if the response carries diagnostics instead of records,
/// or if the record encoding is not supported.
pub fn extract_marc_records(resp: &PresentResponse) -> Result<Vec<Vec<u8>>> {
    let mut out = Vec::new();

    if let Some(Records::ResponseRecords(records)) = &resp.records {
        for rec in records {
            match &rec.record {
                Record::RetrievalRecord(external) => match &external.encoding {
                    ExternalEncoding::OctetAligned(bytes) => {
                        out.push(bytes.to_vec());
                    }
                    ExternalEncoding::SingleASN1Type(any) => {
                        out.push(any.as_bytes().to_vec());
                    }
                    other => {
                        return Err(Error::Protocol(format!("unsupported record encoding: {other:?}")));
                    }
                },
                Record::SurrogateDiagnostic(diag) => {
                    return Err(Error::Protocol(format!("surrogate diagnostic in record: {diag:?}")));
                }
                Record::StartingFragment(f) => {
                    return Err(Error::Protocol(format!("fragmented record not supported (starting fragment): {f:?}")));
                }
                Record::IntermediateFragment(f) => {
                    return Err(Error::Protocol(format!("fragmented record not supported (intermediate fragment): {f:?}")));
                }
                Record::FinalFragment(f) => {
                    return Err(Error::Protocol(format!("fragmented record not supported (final fragment): {f:?}")));
                }
            }
        }
        return Ok(out);
    }

    match &resp.records {
        None => Err(Error::Protocol("present response contains no records".into())),
        Some(Records::NonSurrogateDiagnostic(diag)) => Err(Error::Protocol(format!("present response diagnostic: {diag:?}"))),
        Some(Records::MultipeNonSurDiagnostic(diags)) => Err(Error::Protocol(format!("present response diagnostics: {diags:?}"))),
        Some(other) => Err(Error::Protocol(format!("unexpected records variant in present response: {other:?}"))),
    }
}
