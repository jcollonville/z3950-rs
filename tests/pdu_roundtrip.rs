use z3950_rs::pdu::{Apdu, make_present_request, make_search_request, make_type1_query};

#[test]
fn encode_decode_search_request() {
    let query = make_type1_query(4, "rust").expect("build query");
    let req = make_search_request(&["db1".into()], "rs", query).expect("build request");
    let pdu = Apdu::SearchRequest(req);
    let bytes = rasn::ber::encode(&pdu).expect("encode");
    let decoded: Apdu = rasn::ber::decode(&bytes).expect("decode");
    match decoded {
        Apdu::SearchRequest(decoded_req) => {
            assert_eq!(decoded_req.result_set_name.to_string(), "rs");
        }
        _ => panic!("unexpected PDU"),
    }
}

#[test]
fn encode_decode_present_request() {
    let req = make_present_request("rs", 1, 5).expect("build request");
    let pdu = Apdu::PresentRequest(req);
    let bytes = rasn::ber::encode(&pdu).expect("encode");
    let decoded: Apdu = rasn::ber::decode(&bytes).expect("decode");
    match decoded {
        Apdu::PresentRequest(decoded_req) => {
            assert_eq!(decoded_req.result_set_id.to_string(), "rs");
        }
        _ => panic!("unexpected PDU"),
    }
}
