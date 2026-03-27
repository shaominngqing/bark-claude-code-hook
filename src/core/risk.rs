use serde::{Deserialize, Serialize};
use std::fmt;
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum RiskLevel {
    Low = 0,
    Medium = 1,
    High = 2,
}

impl RiskLevel {
    pub fn from_u8(v: u8) -> Self {
        match v {
            0 => RiskLevel::Low,
            2 => RiskLevel::High,
            _ => RiskLevel::Medium,
        }
    }
}

impl fmt::Display for RiskLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RiskLevel::Low => write!(f, "LOW"),
            RiskLevel::Medium => write!(f, "MEDIUM"),
            RiskLevel::High => write!(f, "HIGH"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AssessmentSource {
    FastRule,
    CustomRule,
    AstAnalysis,
    Cache,
    AI,
    Plugin,
    ChainTracker,
    Fallback,
}

impl fmt::Display for AssessmentSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AssessmentSource::FastRule => write!(f, "FAST"),
            AssessmentSource::CustomRule => write!(f, "RULE"),
            AssessmentSource::AstAnalysis => write!(f, "AST"),
            AssessmentSource::Cache => write!(f, "CACHE"),
            AssessmentSource::AI => write!(f, "AI"),
            AssessmentSource::Plugin => write!(f, "PLUGIN"),
            AssessmentSource::ChainTracker => write!(f, "CHAIN"),
            AssessmentSource::Fallback => write!(f, "FALLBACK"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Assessment {
    pub level: RiskLevel,
    pub reason: String,
    pub source: AssessmentSource,
    #[serde(with = "duration_millis")]
    pub duration: Duration,
}

impl Assessment {
    pub fn new(level: RiskLevel, reason: impl Into<String>, source: AssessmentSource) -> Self {
        Self {
            level,
            reason: reason.into(),
            source,
            duration: Duration::ZERO,
        }
    }

    pub fn with_duration(mut self, d: Duration) -> Self {
        self.duration = d;
        self
    }

    pub fn low(reason: impl Into<String>, source: AssessmentSource) -> Self {
        Self::new(RiskLevel::Low, reason, source)
    }

    pub fn medium(reason: impl Into<String>, source: AssessmentSource) -> Self {
        Self::new(RiskLevel::Medium, reason, source)
    }

    pub fn high(reason: impl Into<String>, source: AssessmentSource) -> Self {
        Self::new(RiskLevel::High, reason, source)
    }
}

mod duration_millis {
    use serde::{Deserialize, Deserializer, Serializer};
    use std::time::Duration;

    pub fn serialize<S: Serializer>(d: &Duration, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_u64(d.as_millis() as u64)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Duration, D::Error> {
        let ms = u64::deserialize(d)?;
        Ok(Duration::from_millis(ms))
    }
}
