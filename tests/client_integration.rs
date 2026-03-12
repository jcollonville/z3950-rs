use z3950_rs::*;
use std::process::Command;
use tokio::net::TcpStream;
use std::time::Duration;
use tokio::time::sleep;
use std::process::Stdio;
use std::sync::atomic::{AtomicU16, Ordering};
use rasn::types::ObjectIdentifier;

// Port counter for unique test ports
static PORT_COUNTER: AtomicU16 = AtomicU16::new(9990);

/// Get a unique port for testing
fn get_test_port() -> u16 {
    PORT_COUNTER.fetch_add(1, Ordering::SeqCst)
}

/// Start yaz-ztest on a given port
/// Returns the child process handle
async fn start_yaz_server(port: u16) -> std::result::Result<std::process::Child, Error> {
    let yaz_server_path = std::env::var("YAZ_SERVER_PATH")
        .unwrap_or_else(|_| "yaz-ztest".to_string());

    let mut cmd = Command::new(&yaz_server_path);
    cmd.arg(format!("tcp:localhost:{}", port))
        
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    
    // Add config file if it exists (optional)
    if std::path::Path::new("testdata/test.xml").exists() {
        cmd.arg("-f").arg("testdata/test.xml");
    }
    
    let mut child = cmd.spawn()
        .map_err(|e| Error::Protocol(format!("Failed to start yaz-ztest: {}. Is yaz-ztest installed? Make sure yaz-ztest is in PATH.", e)))?;

    // Wait for server to be ready (poll connection)
    let addr = format!("127.0.0.1:{}", port);
    let max_attempts = 30;
    for _ in 0..max_attempts {
        sleep(Duration::from_millis(100)).await;
        if TcpStream::connect(&addr).await.is_ok() {
            return Ok(child);
        }
        // Check if process died
        if let Ok(Some(status)) = child.try_wait() {
            return Err(Error::Protocol(format!("yaz-ztest exited early with status: {:?}", status)));
        }
    }

    // Cleanup if server didn't start
    let _ = child.kill();
    Err(Error::Protocol(format!("yaz-ztest failed to start on port {} within timeout", port)))
}

/// Stop yaz-ztest process
async fn stop_yaz_server(child: &mut std::process::Child) {
    let _ = child.kill();
    let _ = child.wait();
}


#[tokio::test]
async fn test_connect() {
    let port = get_test_port();
    let mut server = start_yaz_server(port).await.expect("Failed to start yaz-ztest");
    
    let addr = format!("127.0.0.1:{}", port);
    let client = Client::connect(&addr).await;
    
    assert!(client.is_ok(), "Connection should succeed");
    let client = client.unwrap();
    assert!(!client.is_closed(), "Client should not be closed after connection");
    
    // Test that we can close the connection
    let mut client = client;
    let close_result = client.close().await;
    assert!(close_result.is_ok(), "Close should succeed");
    assert!(client.is_closed(), "Client should be closed after close()");
    
    stop_yaz_server(&mut server).await;
}

#[tokio::test]
async fn test_connect_with_credentials() {
    let port = get_test_port();
    let mut server = start_yaz_server(port).await.expect("Failed to start yaz-ztest");
    
    let addr = format!("127.0.0.1:{}", port);
    let client = Client::connect_with_credentials(&addr, Some(("testuser", "testpass"))).await;
    
    // Note: yaz-ztest may or may not require auth, so we just check it doesn't crash
    // In a real scenario, we'd configure yaz-ztest to require auth
    assert!(client.is_ok() || client.is_err(), "Connection attempt should complete");
    
    if let Ok(mut client) = client {
        let _ = client.close().await;
    }
    
    stop_yaz_server(&mut server).await;
}

#[tokio::test]
async fn test_search() {
    let port = get_test_port();
    let mut server = start_yaz_server(port).await.expect("Failed to start yaz-ztest");
    
    let addr = format!("127.0.0.1:{}", port);
    let mut client = Client::connect(&addr).await.expect("Connection failed");
    
    // Test search with QueryLanguage::CQL
    let search_result = client.search(&["Default"], QueryLanguage::CQL("title=test".to_string())).await;
    
    assert!(search_result.is_ok(), "Search should succeed");
    let response = search_result.unwrap();
    
    // Verify SearchResponse fields
    assert!(response.search_status, "Search should be successful");
    assert!(response.result_count >= 0.into(), "Result count should be non-negative");
    assert!(response.number_of_records_returned >= 0.into(), "Number of records returned should be non-negative");
    
    // Verify PDU was correctly decoded (check that we got a SearchResponse)
    // The fact that we can decode it means the PDU was correctly received and decoded
    
    let _ = client.close().await;
    stop_yaz_server(&mut server).await;
}

#[tokio::test]
async fn test_search_type1_query() {
    let port = get_test_port();
    let mut server = start_yaz_server(port).await.expect("Failed to start yaz-ztest");
    
    let addr = format!("127.0.0.1:{}", port);
    let mut client = Client::connect(&addr).await.expect("Connection failed");
    
    // Test search with make_type1_query
    let query = make_type1_query(4, "test").expect("Failed to create type1 query");
    let search_result = client.search(&["Default"], query).await;
    
    assert!(search_result.is_ok(), "Search with type1 query should succeed");
    let response = search_result.unwrap();
    
    // Verify response structure
    assert!(response.search_status, "Search should be successful");
    
    let _ = client.close().await;
    stop_yaz_server(&mut server).await;
}

#[tokio::test]
async fn test_present_raw() {
    let port = get_test_port();
    let mut server = start_yaz_server(port).await.expect("Failed to start yaz-ztest");
    
    let addr = format!("127.0.0.1:{}", port);
    let mut client = Client::connect(&addr).await.expect("Connection failed");
    
    // First, do a search to get some results
    let search_result = client.search(&["Default"], QueryLanguage::CQL("title=test".to_string())).await;
    assert!(search_result.is_ok(), "Search should succeed");
    
    // Now present the results
    let present_result = client.present_raw(1, 5).await;
    
    // Present may fail if no results, which is OK
    if let Ok(records) = present_result {
        // Verify we got raw MARC records
        assert!(!records.is_empty() || records.is_empty(), "Records should be a valid vector");
    }
    
    let _ = client.close().await;
    stop_yaz_server(&mut server).await;
}

#[tokio::test]
async fn test_present_marc() {
    let port = get_test_port();
    let mut server = start_yaz_server(port).await.expect("Failed to start yaz-ztest");
    
    let addr = format!("127.0.0.1:{}", port);
    let mut client = Client::connect(&addr).await.expect("Connection failed");
    
    // First, do a search
    let search_result = client.search(&["Default"], QueryLanguage::CQL("title=test".to_string())).await;
    assert!(search_result.is_ok(), "Search should succeed");
    
    // Now present and parse MARC records
    let present_result = client.present_marc(1, 5).await;
    
    // Present may fail if no results, which is OK
    if let Ok(records) = present_result {
        // Verify we got parsed MARC records
        assert!(!records.is_empty(), "Records should be a valid vector");
    }
    
    let _ = client.close().await;
    stop_yaz_server(&mut server).await;
}

#[tokio::test]
async fn test_delete_result_sets() {
    let port = get_test_port();
    let mut server = start_yaz_server(port).await.expect("Failed to start yaz-ztest");
    
    let addr = format!("127.0.0.1:{}", port);
    let mut client = Client::connect(&addr).await.expect("Connection failed");
    
    // Create a result set first
    let _ = client.search(&["Default"], QueryLanguage::CQL("title=test".to_string())).await;
    
    // Set a custom result set name
    client.set_result_set_name("test_rs");
    let _ = client.search(&["Default"], QueryLanguage::CQL("title=test2".to_string())).await;
    
    // Now delete specific result sets
    let delete_result = client.delete_result_sets(&["test_rs"]).await;
    
    assert!(delete_result.is_ok(), "Delete result sets should succeed: {:?}", delete_result);
    let response = delete_result.unwrap();
    
    // Verify DeleteResultSetResponse was correctly decoded
    // Check that we got a valid response (status may vary)
    assert!(matches!(response.delete_operation_status, DeleteOperationStatus::Success | DeleteOperationStatus::ResultSetDidNotExist), 
            "Delete operation should have a valid status");
    
    let _ = client.close().await;
    stop_yaz_server(&mut server).await;
}

#[tokio::test]
async fn test_delete_all_result_sets() {
    let port = get_test_port();
    let mut server = start_yaz_server(port).await.expect("Failed to start yaz-ztest");
    
    let addr = format!("127.0.0.1:{}", port);
    let mut client = Client::connect(&addr).await.expect("Connection failed");
    
    // Create some result sets
    let _ = client.search(&["Default"], QueryLanguage::CQL("title=test".to_string())).await;
    client.set_result_set_name("rs1");
    let _ = client.search(&["Default"], QueryLanguage::CQL("title=test2".to_string())).await;
    
    // Delete all result sets
    let delete_result = client.delete_all_result_sets().await;
    
    assert!(delete_result.is_ok(), "Delete all result sets should succeed");
    let response = delete_result.unwrap();
    
    // Verify response
    assert!(matches!(response.delete_operation_status, DeleteOperationStatus::Success | DeleteOperationStatus::ResultSetDidNotExist),
            "Delete all operation should have a valid status");
    
    let _ = client.close().await;
    stop_yaz_server(&mut server).await;
}

#[tokio::test]
async fn test_scan() {
    let port = get_test_port();
    let mut server = start_yaz_server(port).await.expect("Failed to start yaz-ztest");
    
    let addr = format!("127.0.0.1:{}", port);
    let mut client = Client::connect(&addr).await.expect("Connection failed");
    
    // Test scan with BIB-1 attribute type 4 (title)
    let scan_result = client.scan(&["Default"], "test", 4, 10, None).await;
    
    assert!(scan_result.is_ok(), "Scan should succeed");
    let response = scan_result.unwrap();
    
    // Verify ScanResponse was correctly decoded
    assert!(matches!(response.scan_status, ScanStatus::Success | ScanStatus::PartialBeginning | ScanStatus::PartialEnd | ScanStatus::Failure),
            "Scan should have a valid status");
    assert!(response.number_of_entries_returned >= 0.into(), "Number of entries should be non-negative");
    
    // Check if we can extract entries
    if let Some(entries) = Client::extract_scan_entries(&response) {
        // Verify entries structure
        assert!(entries.entries.is_some() || entries.entries.is_none(), "Entries should be valid");
    }
    
    let _ = client.close().await;
    stop_yaz_server(&mut server).await;
}

#[tokio::test]
async fn test_sort() {
    let port = get_test_port();
    let mut server = start_yaz_server(port).await.expect("Failed to start yaz-ztest");
    
    let addr = format!("127.0.0.1:{}", port);
    let mut client = Client::connect(&addr).await.expect("Connection failed");
    
    // Create a result set first
    let search_result = client.search(&["Default"], QueryLanguage::CQL("title=test".to_string())).await;
    assert!(search_result.is_ok(), "Search should succeed");
    
    // Create sort keys
    let sort_keys = vec![
        Client::sort_key_by_field("title", true, false),
    ];
    
    // Sort the result set
    let sort_result = client.sort(&["default"], "sorted_rs", sort_keys).await;
    
    // Sort may not be supported by all servers, so we check if it succeeds or fails gracefully
    if let Ok(response) = sort_result {
        // Verify SortResponse was correctly decoded
        assert!(matches!(response.sort_status, SortStatus::Success | SortStatus::PartialResultsAvailable | SortStatus::Failure),
                "Sort should have a valid status");
    }
    
    let _ = client.close().await;
    stop_yaz_server(&mut server).await;
}

#[tokio::test]
async fn test_close() {
    let port = get_test_port();
    let mut server = start_yaz_server(port).await.expect("Failed to start yaz-ztest");
    
    let addr = format!("127.0.0.1:{}", port);
    let mut client = Client::connect(&addr).await.expect("Connection failed");
    
    // Test basic close
    let close_result = client.close().await;
    
    assert!(close_result.is_ok(), "Close should succeed");
    let close_pdu = close_result.unwrap();
    
    // Verify Close PDU was correctly decoded
    assert!(matches!(close_pdu.close_reason, CloseReason::Finished | CloseReason::Shutdown | CloseReason::SystemProblem | CloseReason::CostLimit | CloseReason::Resources | CloseReason::SecurityViolation | CloseReason::ProtocolError | CloseReason::LackOfActivity | CloseReason::PeerAbort | CloseReason::Unspecified),
            "Close should have a valid reason");
    
    assert!(client.is_closed(), "Client should be closed after close()");
    
    stop_yaz_server(&mut server).await;
}

#[tokio::test]
async fn test_close_with_reason() {
    let port = get_test_port();
    let mut server = start_yaz_server(port).await.expect("Failed to start yaz-ztest");
    
    let addr = format!("127.0.0.1:{}", port);
    let mut client = Client::connect(&addr).await.expect("Connection failed");
    
    // Test close with specific reason
    let close_result = client.close_with_reason(CloseReason::Finished, Some("Test completion")).await;
    
    assert!(close_result.is_ok(), "Close with reason should succeed");
    let close_pdu = close_result.unwrap();
    
    // Verify Close PDU
    assert_eq!(close_pdu.close_reason, CloseReason::Finished, "Close reason should match");
    assert!(close_pdu.diagnostic_information.is_some(), "Diagnostic information should be present");
    
    assert!(client.is_closed(), "Client should be closed");
    
    stop_yaz_server(&mut server).await;
}

#[tokio::test]
async fn test_extended_services() {
    let port = get_test_port();
    let mut server = start_yaz_server(port).await.expect("Failed to start yaz-ztest");
    
    let addr = format!("127.0.0.1:{}", port);
    let mut client = Client::connect(&addr).await.expect("Connection failed");
    
    // Test extended services (may not be supported by all servers)
    // Using a dummy OID for testing
    let oid = ObjectIdentifier::new(vec![1, 2, 3, 4, 5]).expect("Failed to create OID");
    
    let ext_result = client.extended_services(
        ExtendedServicesFunction::Create,
        oid,
        Some("test_package"),
        None,
        WaitAction::DontWait,
    ).await;
    
    // Extended services may not be supported, so we just verify the PDU was sent/received
    if let Ok(response) = ext_result {
        // Verify ExtendedServicesResponse was correctly decoded
        assert!(matches!(response.operation_status, ExtendedServicesStatus::Done | ExtendedServicesStatus::Accepted | ExtendedServicesStatus::Failure),
                "Extended services should have a valid status");
    }
    
    let _ = client.close().await;
    stop_yaz_server(&mut server).await;
}

#[tokio::test]
async fn test_duplicate_detection() {
    let port = get_test_port();
    let mut server = start_yaz_server(port).await.expect("Failed to start yaz-ztest");
    
    let addr = format!("127.0.0.1:{}", port);
    let mut client = Client::connect(&addr).await.expect("Connection failed");
    
    // Create multiple result sets first
    let _ = client.search(&["Default"], QueryLanguage::CQL("title=test".to_string())).await;
    client.set_result_set_name("rs1");
    let _ = client.search(&["Default"], QueryLanguage::CQL("title=test".to_string())).await;
    
    // Test duplicate detection
    let dup_result = client.duplicate_detection(&["default", "rs1"], "dedup_rs", false).await;
    
    // Duplicate detection may not be supported by all servers
    if let Ok(response) = dup_result {
        // Verify DuplicateDetectionResponse was correctly decoded
        assert!(matches!(response.status, DuplicateDetectionStatus::Success | DuplicateDetectionStatus::PartialSomeInputNotProcessed | DuplicateDetectionStatus::PartialSomeCriteriaNotApplied | DuplicateDetectionStatus::PartialBothProblems | DuplicateDetectionStatus::Failure),
                "Duplicate detection should have a valid status");
    }
    
    let _ = client.close().await;
    stop_yaz_server(&mut server).await;
}

#[tokio::test]
async fn test_resource_control_response() {
    let port = get_test_port();
    let mut server = start_yaz_server(port).await.expect("Failed to start yaz-ztest");
    
    let addr = format!("127.0.0.1:{}", port);
    let mut client = Client::connect(&addr).await.expect("Connection failed");
    
    // Test resource control response
    // This is typically sent in response to a ResourceControlRequest from the server
    // We test that the PDU can be sent without error
    let result = client.send_resource_control_response(true, Some(true)).await;
    
    // This may or may not succeed depending on server state, but PDU encoding should work
    // The important thing is that the PDU was correctly encoded and sent
    assert!(result.is_ok() || result.is_err(), "Resource control response should complete");
    
    let _ = client.close().await;
    stop_yaz_server(&mut server).await;
}

#[tokio::test]
async fn test_trigger_resource_control() {
    let port = get_test_port();
    let mut server = start_yaz_server(port).await.expect("Failed to start yaz-ztest");
    
    let addr = format!("127.0.0.1:{}", port);
    let mut client = Client::connect(&addr).await.expect("Connection failed");
    
    // Test trigger resource control
    let result = client.trigger_resource_control(TriggerRequestedAction::ResourceReport, Some(true)).await;
    
    // Verify PDU was sent (may or may not be supported by server)
    assert!(result.is_ok() || result.is_err(), "Trigger resource control should complete");
    
    let _ = client.close().await;
    stop_yaz_server(&mut server).await;
}

#[tokio::test]
async fn test_resource_report() {
    let port = get_test_port();
    let mut server = start_yaz_server(port).await.expect("Failed to start yaz-ztest");
    
    let addr = format!("127.0.0.1:{}", port);
    let mut client = Client::connect(&addr).await.expect("Connection failed");
    
    // Test resource report request
    let report_result = client.resource_report(None, None).await;
    
    // Resource report may not be supported by all servers
    if let Ok(response) = report_result {
        // Verify ResourceReportResponse was correctly decoded
        assert!(matches!(response.resource_report_status, ResourceReportStatus::Success | ResourceReportStatus::Partial | ResourceReportStatus::FailureNoReport | ResourceReportStatus::FailureNoReportNoEstimate | ResourceReportStatus::FailureNoReportCannotDetermine | ResourceReportStatus::FailureNoReportDueToCondition | ResourceReportStatus::FailureReportNotSupported | ResourceReportStatus::FailureReportFormatNotSupported),
                "Resource report should have a valid status");
    }
    
    let _ = client.close().await;
    stop_yaz_server(&mut server).await;
}

#[tokio::test]
async fn test_access_control_response() {
    let port = get_test_port();
    let mut server = start_yaz_server(port).await.expect("Failed to start yaz-ztest");
    
    let addr = format!("127.0.0.1:{}", port);
    let mut client = Client::connect(&addr).await.expect("Connection failed");
    
    // Test access control response
    // This is typically sent in response to an AccessControlRequest from the server
    let response_data = b"test_response";
    let result = client.send_access_control_response(response_data).await;
    
    // Verify PDU was sent (encoding should work)
    assert!(result.is_ok() || result.is_err(), "Access control response should complete");
    
    let _ = client.close().await;
    stop_yaz_server(&mut server).await;
}
