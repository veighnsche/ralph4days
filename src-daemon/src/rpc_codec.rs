use core_errors::{codes, err_string, RalphResult};
use serde::de::DeserializeOwned;
use serde::Serialize;

pub fn require_null_payload(command: &str, payload: serde_json::Value) -> RalphResult<()> {
    if payload.is_null() {
        Ok(())
    } else {
        Err(err_string(
            codes::INTERNAL,
            format!("{command} expects null payload, got: {payload}"),
        ))
    }
}

pub fn decode_args<TArgs: DeserializeOwned>(
    command: &str,
    payload: serde_json::Value,
) -> RalphResult<TArgs> {
    let serde_json::Value::Object(mut map) = payload else {
        return Err(err_string(
            codes::INTERNAL,
            format!("{command} expects payload {{ args: ... }}, got: {payload}"),
        ));
    };

    let args_value = map.remove("args").ok_or_else(|| {
        err_string(
            codes::INTERNAL,
            format!("{command} expects payload {{ args: ... }} (missing 'args' key)"),
        )
    })?;
    if !map.is_empty() {
        let keys = map.keys().cloned().collect::<Vec<_>>().join(", ");
        return Err(err_string(
            codes::INTERNAL,
            format!("{command} payload has unexpected keys: {keys}"),
        ));
    }

    serde_json::from_value(args_value).map_err(|e| {
        err_string(
            codes::INTERNAL,
            format!("{command} args decode failed: {e}"),
        )
    })
}

pub fn encode_result<T: Serialize>(command: &str, value: T) -> RalphResult<serde_json::Value> {
    serde_json::to_value(value).map_err(|e| {
        err_string(
            codes::INTERNAL,
            format!("Failed to encode '{command}' result: {e}"),
        )
    })
}
