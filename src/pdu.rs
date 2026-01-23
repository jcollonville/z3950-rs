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

#[derive(Debug, AsnType, Encode, Decode)]
#[rasn(choice)]
pub enum RecordComposition {
    #[rasn(tag(context, 19))]
    Simple(()),
    #[rasn(tag(context, 209))]
    Complex(()),
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
