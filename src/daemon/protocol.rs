use serde::{Deserialize, Serialize};

use crate::core::protocol::{HookInput, HookOutput};

/// A request sent from a client to the daemon over the Unix domain socket.
///
/// Wire format: one line of JSON terminated by `\n`.
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum DaemonRequest {
    /// Request an assessment for a hook input.
    #[serde(rename = "assess")]
    Assess {
        payload: HookInput,
        session_id: Option<String>,
    },
    /// Query daemon status (uptime, counters, etc.).
    #[serde(rename = "status")]
    Status,
    /// Request a graceful shutdown.
    #[serde(rename = "shutdown")]
    Shutdown,
}

/// A response sent from the daemon back to a client.
///
/// Wire format: one line of JSON terminated by `\n`.
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum DaemonResponse {
    /// Assessment result.
    #[serde(rename = "result")]
    Result {
        payload: HookOutput,
        duration_ms: u64,
    },
    /// Daemon status information.
    #[serde(rename = "status")]
    Status {
        uptime_seconds: u64,
        assessments: u64,
        cache_entries: u64,
    },
    /// An error occurred processing the request.
    #[serde(rename = "error")]
    Error { message: String },
    /// Generic acknowledgement (e.g. for shutdown).
    #[serde(rename = "ok")]
    Ok,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_request_assess_roundtrip() {
        let req = DaemonRequest::Assess {
            payload: HookInput {
                tool_name: "Bash".to_string(),
                tool_input: json!({"command": "ls"}),
            },
            session_id: Some("sess-1".to_string()),
        };
        let json_str = serde_json::to_string(&req).unwrap();
        let parsed: DaemonRequest = serde_json::from_str(&json_str).unwrap();
        match parsed {
            DaemonRequest::Assess { payload, session_id } => {
                assert_eq!(payload.tool_name, "Bash");
                assert_eq!(session_id, Some("sess-1".to_string()));
            }
            _ => panic!("Expected Assess variant"),
        }
    }

    #[test]
    fn test_request_status_roundtrip() {
        let req = DaemonRequest::Status;
        let json_str = serde_json::to_string(&req).unwrap();
        assert!(json_str.contains("\"type\":\"status\""));
    }

    #[test]
    fn test_response_result_roundtrip() {
        let resp = DaemonResponse::Result {
            payload: HookOutput::allow_with_reason("safe"),
            duration_ms: 42,
        };
        let json_str = serde_json::to_string(&resp).unwrap();
        let parsed: DaemonResponse = serde_json::from_str(&json_str).unwrap();
        match parsed {
            DaemonResponse::Result { duration_ms, .. } => {
                assert_eq!(duration_ms, 42);
            }
            _ => panic!("Expected Result variant"),
        }
    }

    #[test]
    fn test_response_error_roundtrip() {
        let resp = DaemonResponse::Error {
            message: "something went wrong".to_string(),
        };
        let json_str = serde_json::to_string(&resp).unwrap();
        let parsed: DaemonResponse = serde_json::from_str(&json_str).unwrap();
        match parsed {
            DaemonResponse::Error { message } => {
                assert_eq!(message, "something went wrong");
            }
            _ => panic!("Expected Error variant"),
        }
    }
}
