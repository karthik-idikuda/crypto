//! Token vesting schedules.

use serde::{Deserialize, Serialize};
use crate::error::TokenomicsError;

/// A single vesting entry for a beneficiary.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VestingEntry {
    pub beneficiary: String,
    pub total_amount: u128,
    pub claimed: u128,
    pub start_time: u64,
    pub cliff_duration: u64,
    pub total_duration: u64,
    pub revoked: bool,
}

impl VestingEntry {
    /// Amount vested at the given timestamp.
    pub fn vested_at(&self, now: u64) -> u128 {
        if self.revoked || now < self.start_time + self.cliff_duration {
            return 0;
        }
        let elapsed = now.saturating_sub(self.start_time);
        if elapsed >= self.total_duration {
            self.total_amount
        } else {
            self.total_amount * (elapsed as u128) / (self.total_duration as u128)
        }
    }

    /// Amount available to claim right now.
    pub fn claimable(&self, now: u64) -> u128 {
        self.vested_at(now).saturating_sub(self.claimed)
    }
}

/// Vesting schedule manager.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VestingSchedule {
    pub entries: Vec<VestingEntry>,
}

impl VestingSchedule {
    pub fn new() -> Self {
        VestingSchedule { entries: Vec::new() }
    }

    /// Add a vesting entry.
    pub fn add_entry(
        &mut self,
        beneficiary: String,
        total_amount: u128,
        start_time: u64,
        cliff_duration: u64,
        total_duration: u64,
    ) {
        self.entries.push(VestingEntry {
            beneficiary,
            total_amount,
            claimed: 0,
            start_time,
            cliff_duration,
            total_duration,
            revoked: false,
        });
    }

    /// Claim vested tokens for a beneficiary.
    pub fn claim(&mut self, beneficiary: &str, now: u64) -> Result<u128, TokenomicsError> {
        let entry = self.entries.iter_mut()
            .find(|e| e.beneficiary == beneficiary && !e.revoked)
            .ok_or(TokenomicsError::StakeNotFound)?;

        let claimable = entry.claimable(now);
        if claimable == 0 {
            return Err(TokenomicsError::VestingLocked);
        }
        entry.claimed = entry.claimed.saturating_add(claimable);
        Ok(claimable)
    }

    /// Revoke a vesting entry. Unclaimed tokens stay locked.
    pub fn revoke(&mut self, beneficiary: &str) -> Result<u128, TokenomicsError> {
        let entry = self.entries.iter_mut()
            .find(|e| e.beneficiary == beneficiary && !e.revoked)
            .ok_or(TokenomicsError::StakeNotFound)?;

        entry.revoked = true;
        Ok(entry.total_amount.saturating_sub(entry.claimed))
    }

    /// Total locked across all entries at given time.
    pub fn total_locked(&self, now: u64) -> u128 {
        self.entries.iter()
            .filter(|e| !e.revoked)
            .map(|e| e.total_amount.saturating_sub(e.vested_at(now)))
            .sum()
    }
}

impl Default for VestingSchedule {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vesting_before_cliff() {
        let mut schedule = VestingSchedule::new();
        schedule.add_entry("alice".into(), 1_000_000, 100, 50, 200);
        let entry = &schedule.entries[0];
        assert_eq!(entry.vested_at(120), 0); // before cliff (100 + 50 = 150)
    }

    #[test]
    fn test_vesting_after_cliff() {
        let mut schedule = VestingSchedule::new();
        schedule.add_entry("alice".into(), 1_000_000, 100, 50, 200);
        let entry = &schedule.entries[0];
        // At time 200, elapsed = 100, vested = 1_000_000 * 100 / 200 = 500_000
        assert_eq!(entry.vested_at(200), 500_000);
    }

    #[test]
    fn test_vesting_fully_vested() {
        let mut schedule = VestingSchedule::new();
        schedule.add_entry("alice".into(), 1_000_000, 100, 50, 200);
        let entry = &schedule.entries[0];
        assert_eq!(entry.vested_at(400), 1_000_000);
    }

    #[test]
    fn test_claim() {
        let mut schedule = VestingSchedule::new();
        schedule.add_entry("alice".into(), 1_000_000, 100, 50, 200);
        let claimed = schedule.claim("alice", 200).unwrap();
        assert_eq!(claimed, 500_000);
        // Second claim at same time yields 0
        assert!(schedule.claim("alice", 200).is_err());
    }

    #[test]
    fn test_revoke() {
        let mut schedule = VestingSchedule::new();
        schedule.add_entry("alice".into(), 1_000_000, 100, 50, 200);
        schedule.claim("alice", 200).unwrap();
        let revoked = schedule.revoke("alice").unwrap();
        assert_eq!(revoked, 500_000); // unclaimed portion
    }

    #[test]
    fn test_total_locked() {
        let mut schedule = VestingSchedule::new();
        schedule.add_entry("alice".into(), 1_000_000, 0, 0, 100);
        schedule.add_entry("bob".into(), 2_000_000, 0, 0, 100);
        // At time 50: each 50% locked
        let locked = schedule.total_locked(50);
        assert_eq!(locked, 1_500_000); // 500k + 1M
    }
}
