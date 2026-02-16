use core_contracts::json_safe::MAX_JSON_SAFE_INTEGER_U64;
use core_contracts::protocol::ProtocolVersionInfo;
use core_contracts::terminal::{PtyClosedEvent, PtyOutputEvent};
use core_contracts::transport::{RemoteEventFrame, RemoteWireFrame};
use core_errors::{codes, RalphError};

#[test]
fn contract_suite_protocol_version_info_serializes_expected_shape() {
    let json = serde_json::to_value(ProtocolVersionInfo::current()).expect("serialize");
    assert!(
        json.get("protocolVersion").is_some(),
        "protocolVersion key should exist (camelCase wire shape)"
    );
}

#[test]
fn contract_suite_pty_events_serialize_expected_shape() {
    let output = PtyOutputEvent {
        session_id: "s".to_owned(),
        seq: 1,
        data: "AA==".to_owned(),
    };
    let json = serde_json::to_value(output).expect("serialize");
    assert_eq!(json["sessionId"], "s");
    assert_eq!(json["seq"], 1);
    assert_eq!(json["data"], "AA==");

    let closed = PtyClosedEvent {
        session_id: "s".to_owned(),
        exit_code: 0,
    };
    let json = serde_json::to_value(closed).expect("serialize");
    assert_eq!(json["sessionId"], "s");
    assert_eq!(json["exitCode"], 0);
}

#[test]
fn contract_suite_remote_wire_frames_are_strict_and_json_safe() {
    let frame = RemoteWireFrame::RpcRequest {
        id: 1,
        command: "cmd".to_owned(),
        payload: serde_json::Value::Null,
    };
    let text = serde_json::to_string(&frame).expect("serialize");
    let decoded: RemoteWireFrame = serde_json::from_str(&text).expect("decode");
    assert!(matches!(decoded, RemoteWireFrame::RpcRequest { .. }));

    let too_large = RemoteWireFrame::RpcRequest {
        id: MAX_JSON_SAFE_INTEGER_U64 + 1,
        command: "cmd".to_owned(),
        payload: serde_json::Value::Null,
    };
    let err = serde_json::to_string(&too_large).unwrap_err();
    assert!(
        err.to_string().contains("JSON-safe"),
        "expected JSON-safe integer enforcement error, got: {err}"
    );

    let unknown_field = serde_json::json!({
        "type": "rpc-ok",
        "id": 1,
        "result": null,
        "extra": 1
    });
    let err = serde_json::from_value::<RemoteWireFrame>(unknown_field).unwrap_err();
    assert!(
        err.to_string().contains("unknown field"),
        "expected strict decode error, got: {err}"
    );
}

#[test]
fn contract_suite_remote_event_frames_roundtrip() {
    let ev = RemoteEventFrame::TerminalOutput(PtyOutputEvent {
        session_id: "s".to_owned(),
        seq: 1,
        data: "AA==".to_owned(),
    });
    let json = serde_json::to_value(&ev).expect("serialize");
    assert_eq!(json["event"], "terminal:output");
}

#[test]
fn contract_suite_structured_error_shape_is_stable() {
    let err = RalphError::new(codes::INTERNAL, "boom".to_owned());
    let json = serde_json::to_value(&err).expect("serialize");
    assert_eq!(json["code"], codes::INTERNAL);
    assert_eq!(json["message"], "boom");
    assert!(json.get("location").is_some());
    assert!(json["location"]["file"]
        .as_str()
        .unwrap()
        .ends_with("contract_suite_test.rs"));
    assert!(json["location"]["line"].as_u64().unwrap() > 0);
    assert!(json["location"]["column"].as_u64().unwrap() > 0);
    assert!(json["context"].as_array().unwrap().is_empty());
    assert!(json["hint"].is_null());
}
