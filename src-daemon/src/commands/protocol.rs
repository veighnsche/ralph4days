use ralph_contracts::protocol::ProtocolVersionInfo;
use ralph_errors::RalphResult;

use crate::rpc_codec::{encode_result, require_null_payload};

pub fn protocol_version_get(payload: serde_json::Value) -> RalphResult<serde_json::Value> {
    require_null_payload("protocol_version_get", payload)?;
    encode_result("protocol_version_get", ProtocolVersionInfo::current())
}
