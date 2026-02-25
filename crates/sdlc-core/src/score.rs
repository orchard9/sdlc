use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Severity
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    Critical,
    High,
    Medium,
    Low,
}

impl Severity {
    pub fn default_points(self) -> u32 {
        match self {
            Severity::Critical => 20,
            Severity::High => 10,
            Severity::Medium => 5,
            Severity::Low => 2,
        }
    }
}

// ---------------------------------------------------------------------------
// Deduction
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Deduction {
    pub severity: Severity,
    pub points: u32,
    pub description: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,
}

// ---------------------------------------------------------------------------
// QualityScore
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityScore {
    /// Lens name: "product_fit", "research_grounding", "implementation", etc.
    pub lens: String,
    /// Score from 0 to 100.
    pub score: u32,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub deductions: Vec<Deduction>,
    /// Agent ID that produced this score.
    pub evaluator: String,
    pub timestamp: String,
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn severity_default_points() {
        assert_eq!(Severity::Critical.default_points(), 20);
        assert_eq!(Severity::High.default_points(), 10);
        assert_eq!(Severity::Medium.default_points(), 5);
        assert_eq!(Severity::Low.default_points(), 2);
    }

    #[test]
    fn quality_score_yaml_roundtrip() {
        let score = QualityScore {
            lens: "product_fit".to_string(),
            score: 85,
            deductions: vec![Deduction {
                severity: Severity::Medium,
                points: 5,
                description: "Missing edge case coverage".to_string(),
                location: Some("spec.md:42".to_string()),
            }],
            evaluator: "review-agent".to_string(),
            timestamp: "2026-02-24T00:00:00Z".to_string(),
        };
        let yaml = serde_yaml::to_string(&score).unwrap();
        let parsed: QualityScore = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(parsed.lens, "product_fit");
        assert_eq!(parsed.score, 85);
        assert_eq!(parsed.deductions.len(), 1);
        assert_eq!(parsed.deductions[0].severity, Severity::Medium);
        assert_eq!(parsed.deductions[0].points, 5);
        assert_eq!(parsed.deductions[0].location.as_deref(), Some("spec.md:42"));
        assert_eq!(parsed.evaluator, "review-agent");
    }

    #[test]
    fn quality_score_json_roundtrip() {
        let score = QualityScore {
            lens: "implementation".to_string(),
            score: 92,
            deductions: vec![],
            evaluator: "audit-agent".to_string(),
            timestamp: "2026-02-24T12:00:00Z".to_string(),
        };
        let json = serde_json::to_string(&score).unwrap();
        let parsed: QualityScore = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.lens, "implementation");
        assert_eq!(parsed.score, 92);
        assert!(parsed.deductions.is_empty());
    }

    #[test]
    fn deduction_roundtrip() {
        let d = Deduction {
            severity: Severity::Critical,
            points: 20,
            description: "Security vulnerability".to_string(),
            location: None,
        };
        let yaml = serde_yaml::to_string(&d).unwrap();
        let parsed: Deduction = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(parsed.severity, Severity::Critical);
        assert_eq!(parsed.points, 20);
        assert!(parsed.location.is_none());
    }

    #[test]
    fn deduction_with_location_roundtrip() {
        let d = Deduction {
            severity: Severity::Low,
            points: 2,
            description: "Minor style issue".to_string(),
            location: Some("main.rs:10".to_string()),
        };
        let json = serde_json::to_string(&d).unwrap();
        let parsed: Deduction = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.location.as_deref(), Some("main.rs:10"));
    }

    #[test]
    fn severity_serde_roundtrip() {
        for &sev in &[
            Severity::Critical,
            Severity::High,
            Severity::Medium,
            Severity::Low,
        ] {
            let json = serde_json::to_string(&sev).unwrap();
            let parsed: Severity = serde_json::from_str(&json).unwrap();
            assert_eq!(parsed, sev);
        }
    }
}
