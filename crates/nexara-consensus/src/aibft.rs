//! AI-Enhanced Byzantine Fault Tolerance (AIBFT) module.
//!
//! Detects and responds to Byzantine behavior using anomaly detection
//! heuristics. Adjusts validator trust scores based on observed behavior.

use nexara_crypto::WalletAddress;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

/// A behavioral observation of a validator.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorObservation {
    pub validator: WalletAddress,
    pub block_height: u64,
    pub response_time_ms: u64,
    pub voted_correctly: bool,
    pub participated: bool,
}

/// Trust score tracking for validators.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustScore {
    pub score: f64,
    pub observations: u64,
    pub correct_votes: u64,
    pub missed_rounds: u64,
    pub anomaly_count: u64,
}

impl Default for TrustScore {
    fn default() -> Self {
        TrustScore {
            score: 1.0,
            observations: 0,
            correct_votes: 0,
            missed_rounds: 0,
            anomaly_count: 0,
        }
    }
}

/// The AIBFT anomaly detection engine.
pub struct AiBft {
    pub trust_scores: HashMap<WalletAddress, TrustScore>,
    /// Average response time used as baseline.
    pub avg_response_time_ms: u64,
    /// Multiplier threshold: if response_time > avg * threshold_multiplier, it's anomalous.
    pub threshold_multiplier: f64,
    /// Minimum trust score before a validator is flagged.
    pub min_trust_score: f64,
}

impl AiBft {
    /// Create a new AIBFT engine.
    pub fn new() -> Self {
        AiBft {
            trust_scores: HashMap::new(),
            avg_response_time_ms: 100,
            threshold_multiplier: 3.0,
            min_trust_score: 0.3,
        }
    }

    /// Record an observation and update trust scores.
    pub fn record_observation(&mut self, obs: ValidatorObservation) {
        let entry = self.trust_scores.entry(obs.validator).or_default();
        entry.observations += 1;

        if obs.voted_correctly {
            entry.correct_votes += 1;
        }

        if !obs.participated {
            entry.missed_rounds += 1;
        }

        // Check for anomalous response time
        if obs.response_time_ms > self.avg_response_time_ms * self.threshold_multiplier as u64 {
            entry.anomaly_count += 1;
        }

        // Recalculate trust score
        let score = Self::calculate_trust_static(entry);
        entry.score = score;
    }

    /// Static version that doesn't borrow self.
    fn calculate_trust_static(ts: &TrustScore) -> f64 {
        if ts.observations == 0 {
            return 1.0;
        }

        let correctness = ts.correct_votes as f64 / ts.observations as f64;
        let participation = 1.0 - (ts.missed_rounds as f64 / ts.observations as f64);
        let anomaly_penalty = 1.0 - (ts.anomaly_count as f64 / ts.observations as f64).min(0.5);

        // Weighted combination
        let score = correctness * 0.4 + participation * 0.4 + anomaly_penalty * 0.2;
        score.clamp(0.0, 1.0)
    }

    /// Get the trust score for a validator.
    pub fn get_trust_score(&self, validator: &WalletAddress) -> f64 {
        self.trust_scores.get(validator)
            .map(|ts| ts.score)
            .unwrap_or(1.0)
    }

    /// Get all validators below the minimum trust threshold.
    pub fn get_untrusted_validators(&self) -> Vec<(WalletAddress, f64)> {
        self.trust_scores.iter()
            .filter(|(_, ts)| ts.score < self.min_trust_score)
            .map(|(addr, ts)| (*addr, ts.score))
            .collect()
    }

    /// Detect if a specific observation is anomalous.
    pub fn is_anomalous(&self, obs: &ValidatorObservation) -> bool {
        if !obs.participated {
            return true;
        }
        if !obs.voted_correctly {
            return true;
        }
        obs.response_time_ms > self.avg_response_time_ms * self.threshold_multiplier as u64
    }

    /// Reset trust scores (e.g., at epoch boundary).
    pub fn reset_scores(&mut self) {
        self.trust_scores.clear();
    }
}

impl Default for AiBft {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_validator_trust() {
        let aibft = AiBft::new();
        let addr = WalletAddress::zero();
        assert_eq!(aibft.get_trust_score(&addr), 1.0);
    }

    #[test]
    fn test_good_behavior() {
        let mut aibft = AiBft::new();
        let addr = WalletAddress::zero();
        for i in 0..10 {
            aibft.record_observation(ValidatorObservation {
                validator: addr,
                block_height: i,
                response_time_ms: 50,
                voted_correctly: true,
                participated: true,
            });
        }
        assert!(aibft.get_trust_score(&addr) > 0.9);
    }

    #[test]
    fn test_bad_behavior_lowers_trust() {
        let mut aibft = AiBft::new();
        let addr = WalletAddress::zero();
        for i in 0..10 {
            aibft.record_observation(ValidatorObservation {
                validator: addr,
                block_height: i,
                response_time_ms: 500,
                voted_correctly: false,
                participated: false,
            });
        }
        assert!(aibft.get_trust_score(&addr) < 0.5);
    }

    #[test]
    fn test_anomaly_detection() {
        let aibft = AiBft::new();
        let obs = ValidatorObservation {
            validator: WalletAddress::zero(),
            block_height: 1,
            response_time_ms: 1000,
            voted_correctly: true,
            participated: true,
        };
        assert!(aibft.is_anomalous(&obs));
    }

    #[test]
    fn test_untrusted_validators() {
        let mut aibft = AiBft::new();
        let addr = WalletAddress::zero();
        for i in 0..20 {
            aibft.record_observation(ValidatorObservation {
                validator: addr,
                block_height: i,
                response_time_ms: 500,
                voted_correctly: false,
                participated: false,
            });
        }
        let untrusted = aibft.get_untrusted_validators();
        assert!(!untrusted.is_empty());
    }
}
