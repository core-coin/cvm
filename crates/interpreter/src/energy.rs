mod calc;
mod constants;

pub use calc::*;
pub use constants::*;

#[derive(Clone, Copy, Debug)]
pub struct Energy {
    /// energy Limit
    limit: u64,
    /// used+memory energy.
    all_used_energy: u64,
    /// Used energy without memory
    used: u64,
    /// Used energy for memory expansion
    memory: u64,
    /// Refunded energy. This energy is used only at the end of execution.
    refunded: i64,
}
impl Energy {
    pub fn new(limit: u64) -> Self {
        Self {
            limit,
            used: 0,
            memory: 0,
            refunded: 0,
            all_used_energy: 0,
        }
    }

    pub fn limit(&self) -> u64 {
        self.limit
    }

    pub fn memory(&self) -> u64 {
        self.memory
    }

    pub fn refunded(&self) -> i64 {
        self.refunded
    }

    pub fn spend(&self) -> u64 {
        self.all_used_energy
    }

    pub fn remaining(&self) -> u64 {
        self.limit - self.all_used_energy
    }

    pub fn erase_cost(&mut self, returned: u64) {
        self.used -= returned;
        self.all_used_energy -= returned;
    }

    pub fn record_refund(&mut self, refund: i64) {
        self.refunded += refund;
    }

    /// Record an explicit cost.
    #[inline(always)]
    pub fn record_cost(&mut self, cost: u64) -> bool {
        let (all_used_energy, overflow) = self.all_used_energy.overflowing_add(cost);
        if overflow || self.limit < all_used_energy {
            return false;
        }

        self.used += cost;
        self.all_used_energy = all_used_energy;
        true
    }

    /// used in memory_resize! macro to record energy used for memory expansion.
    pub fn record_memory(&mut self, energy_memory: u64) -> bool {
        if energy_memory > self.memory {
            let (all_used_energy, overflow) = self.used.overflowing_add(energy_memory);
            if overflow || self.limit < all_used_energy {
                return false;
            }
            self.memory = energy_memory;
            self.all_used_energy = all_used_energy;
        }
        true
    }

    /// used in energy_refund! macro to record refund value.
    /// Refund can be negative but self.refunded is always positive.
    pub fn energy_refund(&mut self, refund: i64) {
        self.refunded += refund;
    }
}
