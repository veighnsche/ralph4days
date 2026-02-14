use ralph_macros::ipc_type;
use serde::{Deserialize, Serialize};

/// Bump this when the *frontend-facing* IPC contract changes (commands/events/payload shapes).
///
/// Remote clients must hard-fail on mismatch with a deterministic upgrade path.
pub const PROTOCOL_VERSION: u32 = 1;

#[ipc_type]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProtocolVersionInfo {
    pub protocol_version: u32,
}

impl ProtocolVersionInfo {
    pub fn current() -> Self {
        Self {
            protocol_version: PROTOCOL_VERSION,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn protocol_version_is_stable() {
        assert_eq!(PROTOCOL_VERSION, 1);
    }

    #[test]
    fn protocol_version_info_serializes_expected_shape() {
        let json = serde_json::to_value(ProtocolVersionInfo::current()).unwrap();
        assert_eq!(json["protocolVersion"], 1);
    }
}
