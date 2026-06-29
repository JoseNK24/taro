use serde_json::Value;
use std::io::Write;

const LOG_PATH: &str = "/Users/jose/Documents/Repos/.cursor/debug-da2dd5.log";
const SESSION_ID: &str = "da2dd5";

// #region agent log
pub fn agent_debug_log(hypothesis_id: &str, location: &str, message: &str, data: Value) {
    let payload = serde_json::json!({
        "sessionId": SESSION_ID,
        "hypothesisId": hypothesis_id,
        "location": location,
        "message": message,
        "data": data,
        "timestamp": std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0),
    });
    if let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(LOG_PATH)
    {
        let _ = writeln!(file, "{payload}");
    }
    eprintln!("[taro-debug] {payload}");
}
// #endregion
