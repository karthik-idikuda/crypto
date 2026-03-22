//! Solana bridge adapter.

use serde::{Deserialize, Serialize};
use crate::error::BridgeError;

/// Solana bridge configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolanaBridge {
    pub program_id: String,
    pub confirmations_required: u32,
    pub enabled: bool,
}

/// Solana deposit event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolanaDepositEvent {
    pub signature: String,
    pub sender: String,
    pub recipient: String,
    pub amount: u128,
    pub slot: u64,
}

impl SolanaBridge {
    pub fn new(program_id: &str) -> Self {
        SolanaBridge {
            program_id: program_id.to_string(),
            confirmations_required: 32,
            enabled: true,
        }
    }

    pub fn validate_deposit(&self, event: &SolanaDepositEvent) -> Result<bool, BridgeError> {
        if !self.enabled {
            return Err(BridgeError::BridgePaused("Solana".into()));
        }
        if event.amount == 0 {
            return Err(BridgeError::InvalidMessage("Zero amount".into()));
        }
        Ok(!event.signature.is_empty())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_solana_validate() {
        let bridge = SolanaBridge::new("SoLProgramId123");
        let event = SolanaDepositEvent {
            signature: "sig123".into(),
            sender: "s".into(),
            recipient: "r".into(),
            amount: 100,
            slot: 50,
        };
        assert!(bridge.validate_deposit(&event).unwrap());
    }
}
