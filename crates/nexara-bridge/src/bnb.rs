//! BNB Chain bridge adapter.

use serde::{Deserialize, Serialize};
use crate::error::BridgeError;

/// BNB Chain bridge configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BnbBridge {
    pub contract_address: String,
    pub chain_id: u64,
    pub confirmations_required: u32,
    pub enabled: bool,
}

/// BNB deposit event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BnbDepositEvent {
    pub tx_hash: String,
    pub sender: String,
    pub recipient: String,
    pub amount: u128,
    pub block_number: u64,
}

impl BnbBridge {
    pub fn new(contract_address: &str) -> Self {
        BnbBridge {
            contract_address: contract_address.to_string(),
            chain_id: 56,
            confirmations_required: 15,
            enabled: true,
        }
    }

    pub fn validate_deposit(&self, event: &BnbDepositEvent) -> Result<bool, BridgeError> {
        if !self.enabled {
            return Err(BridgeError::BridgePaused("BNB".into()));
        }
        if event.amount == 0 {
            return Err(BridgeError::InvalidMessage("Zero amount".into()));
        }
        Ok(!event.tx_hash.is_empty())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bnb_validate() {
        let bridge = BnbBridge::new("0xcontract");
        let event = BnbDepositEvent {
            tx_hash: "0xabc".into(),
            sender: "s".into(),
            recipient: "r".into(),
            amount: 100,
            block_number: 1,
        };
        assert!(bridge.validate_deposit(&event).unwrap());
    }
}
