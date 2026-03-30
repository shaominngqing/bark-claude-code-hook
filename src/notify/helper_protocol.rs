use serde::{Deserialize, Serialize};

/// Request from bark daemon to the notification helper app.
///
/// Wire format: one-line JSON terminated by `\n`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum NotifyRequest {
    /// Informational notification (Medium risk, no response needed).
    #[serde(rename = "info")]
    Info { title: String, body: String },

    /// Confirmation notification (High risk, response required).
    ///
    /// The helper should display a notification with Allow/Deny/Skip buttons
    /// and respond with a `Decision`.
    #[serde(rename = "confirm")]
    Confirm {
        id: String,
        title: String,
        body: String,
        reason: String,
    },

    /// Health check.
    #[serde(rename = "ping")]
    Ping,
}

/// Response from the notification helper app back to bark daemon.
///
/// Wire format: one-line JSON terminated by `\n`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum NotifyResponse {
    /// Info notification was displayed.
    #[serde(rename = "ack")]
    Ack,

    /// User made a decision on a Confirm notification.
    #[serde(rename = "decision")]
    Decision { id: String, action: DecisionAction },

    /// Pong reply to Ping.
    #[serde(rename = "pong")]
    Pong,

    /// Helper encountered an error.
    #[serde(rename = "error")]
    Error { message: String },
}

/// The user's decision from the notification buttons.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DecisionAction {
    /// User clicked "Allow" — bark returns permissionDecision: allow.
    Allow,
    /// User clicked "Deny" — bark returns permissionDecision: deny.
    Deny,
    /// User clicked "Skip", dismissed the notification, or it timed out
    /// — bark returns permissionDecision: ask (fallback to terminal).
    Skip,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_notify_request_info_roundtrip() {
        let req = NotifyRequest::Info {
            title: "Bark".into(),
            body: "Auto-allowed".into(),
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains(r#""type":"info"#));
        let parsed: NotifyRequest = serde_json::from_str(&json).unwrap();
        match parsed {
            NotifyRequest::Info { title, body } => {
                assert_eq!(title, "Bark");
                assert_eq!(body, "Auto-allowed");
            }
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn test_notify_request_confirm_roundtrip() {
        let req = NotifyRequest::Confirm {
            id: "abc-123".into(),
            title: "High Risk".into(),
            body: "rm -rf /".into(),
            reason: "Destructive command".into(),
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains(r#""type":"confirm"#));
        let parsed: NotifyRequest = serde_json::from_str(&json).unwrap();
        match parsed {
            NotifyRequest::Confirm { id, .. } => assert_eq!(id, "abc-123"),
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn test_notify_response_decision_roundtrip() {
        let resp = NotifyResponse::Decision {
            id: "abc-123".into(),
            action: DecisionAction::Allow,
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains(r#""action":"allow"#));
        let parsed: NotifyResponse = serde_json::from_str(&json).unwrap();
        match parsed {
            NotifyResponse::Decision { id, action } => {
                assert_eq!(id, "abc-123");
                assert_eq!(action, DecisionAction::Allow);
            }
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn test_ping_pong() {
        let ping = serde_json::to_string(&NotifyRequest::Ping).unwrap();
        assert!(ping.contains(r#""type":"ping"#));
        let pong = serde_json::to_string(&NotifyResponse::Pong).unwrap();
        assert!(pong.contains(r#""type":"pong"#));
    }
}
