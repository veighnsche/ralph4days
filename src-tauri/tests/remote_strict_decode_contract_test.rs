use core_contracts::agent_sessions::AgentSessionsByIdArgs;
use core_contracts::domain::TaskSignalSummary;
use core_contracts::project::{ProjectInfo, ProjectScanArgs, RalphProject, RecentProject};
use core_contracts::remote::{
    RemoteConnectResult, RemoteSshConnectResult, RemoteSshHostKeyChallenge, RemoteSshProfile,
    RemoteSshProfileUpsertArgs, RemoteSshStatus,
};
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
    assert_rejects_unknown_fields::<RemoteSshConnectResult>(serde_json::json!({
        "wsUrl": "ws://127.0.0.1:19444",
        "protocol": { "protocolVersion": 1 },
        "sshSessionId": 1234,
        "host": "example.local",
        "username": "vince",
        "sshPort": 22,
        "remotePort": 9944,
        "identityFile": null,
        "knownHostsFile": "/Users/vince/.ssh/known_hosts",
        "unknown": 1
    }));
    assert_rejects_unknown_fields::<RemoteSshStatus>(serde_json::json!({
        "active": true,
        "wsUrl": "ws://127.0.0.1:19444",
        "sshSessionId": 1234,
        "activeProfileId": "sshp-1",
        "host": "example.local",
        "username": "vince",
        "sshPort": 22,
        "remotePort": 9944,
        "authMode": "key",
        "transportKind": "stream",
        "identityFile": null,
        "knownHostsFile": "/Users/vince/.ssh/known_hosts",
        "unknown": 1
    }));
    assert_rejects_unknown_fields::<RemoteSshProfile>(serde_json::json!({
        "id": "sshp-1",
        "name": "Home",
        "host": "example.local",
        "username": "vince",
        "sshPort": 22,
        "remotePort": 9944,
        "authMode": "key",
        "identityFile": null,
        "identityRef": null,
        "knownHostsFile": null,
        "autoReconnectEnabled": false,
        "lastUsedAt": null,
        "unknown": 1
    }));
    assert_rejects_unknown_fields::<RemoteSshProfileUpsertArgs>(serde_json::json!({
        "id": null,
        "name": "Home",
        "host": "example.local",
        "username": "vince",
        "sshPort": 22,
        "remotePort": 9944,
        "authMode": "key",
        "identityFile": null,
        "identityRef": null,
        "knownHostsFile": null,
        "autoReconnectEnabled": false,
        "password": null,
        "keyPassphrase": null,
        "savePassword": false,
        "saveKeyPassphrase": false,
        "unknown": 1
    }));
    assert_rejects_unknown_fields::<RemoteSshHostKeyChallenge>(serde_json::json!({
        "challengeId": "sshhk-1",
        "host": "example.local",
        "sshPort": 22,
        "algorithm": "ssh-ed25519",
        "fingerprintSha256": "SHA256:abc",
        "knownHostsTargetPath": "/Users/vince/.ssh/known_hosts",
        "expiresAt": "2026-02-16T00:00:00Z",
        "unknown": 1
    }));

    // ralph-backend result DTO
    assert_rejects_unknown_fields::<RecentProject>(serde_json::json!({
        "path": "/tmp/p",
        "name": "p",
        "last_opened": "2026-02-15T00:00:00Z",
        "unknown": 1
    }));

    // domain result DTO
    assert_rejects_unknown_fields::<TaskSignalSummary>(serde_json::json!({
        "pendingAsks": 1,
        "flagCount": 0,
        "maxFlagSeverity": null,
        "lastClosingVerb": null,
        "sessionCount": 0,
        "learnedCount": 0,
        "unknown": 1
    }));
}
