//! Cosmos/IBC bridge adapter.

use serde::{Deserialize, Serialize};
use crate::error::BridgeError;

/// Cosmos IBC bridge configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CosmosBridge {
    pub channel_id: String,
    pub port_id: String,
    pub enabled: bool,
}

/// IBC packet event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IbcPacketEvent {
    pub sequence: u64,
    pub source_channel: String,
    pub dest_channel: String,
    pub sender: String,
    pub recipient: String,
    pub amount: u128,
    pub denom: String,
}

impl CosmosBridge {
    pub fn new(channel_id: &str) -> Self {
        CosmosBridge {
            channel_id: channel_id.to_string(),
            port_id: "transfer".to_string(),
            enabled: true,
        }
    }

    pub fn validate_packet(&self, packet: &IbcPacketEvent) -> Result<bool, BridgeError> {
        if !self.enabled {
            return Err(BridgeError::BridgePaused("Cosmos".into()));
        }
        if packet.amount == 0 {
            return Err(BridgeError::InvalidMessage("Zero amount".into()));
        }
        Ok(packet.dest_channel == self.channel_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cosmos_validate() {
        let bridge = CosmosBridge::new("channel-0");
        let packet = IbcPacketEvent {
            sequence: 1,
            source_channel: "channel-1".into(),
            dest_channel: "channel-0".into(),
            sender: "cosmos1sender".into(),
            recipient: "nxr_recipient".into(),
            amount: 500,
            denom: "uatom".into(),
        };
        assert!(bridge.validate_packet(&packet).unwrap());
    }
}
