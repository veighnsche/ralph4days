use ralph4days_lib::ipc_contract::{
    AgentSessionsByIdArgs, ProjectInfo, ProjectScanArgs, RalphProject, RemoteConnectResult,
};
use ralph_backend::project_contract::RecentProject;
use serde::de::DeserializeOwned;

#[test]
fn strict_decode_rejects_unknown_fields_in_request_and_result_dtos() {
    fn assert_rejects_unknown_fields<T: DeserializeOwned>(v: serde_json::Value) {
        let Err(err) = serde_json::from_value::<T>(v) else {
            panic!("expected strict decode failure due to unknown fields");
        };
        assert!(
            err.to_string().contains("unknown field"),
            "expected unknown field error, got: {err}"
        );
    }

    // src-tauri DTOs
    assert_rejects_unknown_fields::<ProjectScanArgs>(
        serde_json::json!({ "rootDir": null, "unknown": 1 }),
    );
    assert_rejects_unknown_fields::<AgentSessionsByIdArgs>(
        serde_json::json!({ "id": "abc", "unknown": 1 }),
    );
    assert_rejects_unknown_fields::<RalphProject>(
        serde_json::json!({ "name": "p", "path": "/tmp/p", "unknown": 1 }),
    );
    assert_rejects_unknown_fields::<ProjectInfo>(
        serde_json::json!({ "title": "t", "description": null, "created": null, "unknown": 1 }),
    );
    assert_rejects_unknown_fields::<RemoteConnectResult>(serde_json::json!({
        "wsUrl": "ws://localhost:1234",
        "protocol": { "protocolVersion": 1 },
        "unknown": 1
    }));

    // ralph-backend result DTO
    assert_rejects_unknown_fields::<RecentProject>(serde_json::json!({
        "path": "/tmp/p",
        "name": "p",
        "last_opened": "2026-02-15T00:00:00Z",
        "unknown": 1
    }));

    // sqlite-db result DTO
    assert_rejects_unknown_fields::<sqlite_db::TaskSignalSummary>(serde_json::json!({
        "pendingAsks": 1,
        "flagCount": 0,
        "maxFlagSeverity": null,
        "lastClosingVerb": null,
        "sessionCount": 0,
        "learnedCount": 0,
        "unknown": 1
    }));
}
