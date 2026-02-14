use ralph_contracts::protocol::ProtocolVersionInfo;

#[tauri::command]
pub fn protocol_version_get() -> ProtocolVersionInfo {
    ProtocolVersionInfo::current()
}
