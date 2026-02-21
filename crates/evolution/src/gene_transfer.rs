use chrono::{DateTime, Utc};
use genesis_dna::AgentID;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::errors::EvolutionError;

/// Maximum size of a gene module (code snippet) in bytes.
pub const MAX_MODULE_SIZE: usize = 1_048_576; // 1 MB

/// A shareable code module (DNA snippet) for horizontal gene transfer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneModule {
    /// Unique module ID.
    pub id: Uuid,
    /// Human-readable name.
    pub name: String,
    /// Description of what this module does.
    pub description: String,
    /// The code/data payload.
    pub payload: Vec<u8>,
    /// SHA-256 hash of the payload.
    pub integrity_hash: [u8; 32],
    /// The agent that created this module.
    pub creator: AgentID,
    /// ATP asking price for this module.
    pub price: f64,
    /// Number of times this module has been transferred.
    pub transfer_count: u64,
    /// Average fitness improvement reported by adopters.
    pub avg_fitness_impact: f64,
    /// Created timestamp.
    pub created_at: DateTime<Utc>,
}

impl GeneModule {
    /// Create a new gene module.
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        payload: Vec<u8>,
        creator: AgentID,
        price: f64,
    ) -> Result<Self, EvolutionError> {
        if payload.len() > MAX_MODULE_SIZE {
            return Err(EvolutionError::ModuleTooLarge {
                size: payload.len(),
                max: MAX_MODULE_SIZE,
            });
        }

        let mut hasher = Sha256::new();
        hasher.update(&payload);
        let hash = hasher.finalize();
        let mut integrity_hash = [0u8; 32];
        integrity_hash.copy_from_slice(&hash);

        Ok(Self {
            id: Uuid::new_v4(),
            name: name.into(),
            description: description.into(),
            payload,
            integrity_hash,
            creator,
            price,
            transfer_count: 0,
            avg_fitness_impact: 0.0,
            created_at: Utc::now(),
        })
    }

    /// Verify the integrity of the module payload.
    pub fn verify_integrity(&self) -> bool {
        let mut hasher = Sha256::new();
        hasher.update(&self.payload);
        let hash = hasher.finalize();
        hash.as_slice() == self.integrity_hash
    }

    /// Record a transfer and associated fitness impact.
    pub fn record_transfer(&mut self, fitness_delta: f64) {
        let total = self.avg_fitness_impact * self.transfer_count as f64;
        self.transfer_count += 1;
        self.avg_fitness_impact = (total + fitness_delta) / self.transfer_count as f64;
    }
}

/// An offer to transfer a gene module between agents.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneTransferOffer {
    /// Unique offer ID.
    pub id: Uuid,
    /// The seller agent.
    pub seller: AgentID,
    /// The buyer agent.
    pub buyer: AgentID,
    /// The module being offered.
    pub module_id: Uuid,
    /// Agreed price (ATP).
    pub price: f64,
    /// Whether the transfer has been completed.
    pub completed: bool,
    /// Timestamp.
    pub created_at: DateTime<Utc>,
}

impl GeneTransferOffer {
    pub fn new(seller: AgentID, buyer: AgentID, module_id: Uuid, price: f64) -> Self {
        Self {
            id: Uuid::new_v4(),
            seller,
            buyer,
            module_id,
            price,
            completed: false,
            created_at: Utc::now(),
        }
    }

    pub fn complete(&mut self) {
        self.completed = true;
    }
}

/// Marketplace for gene modules.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct GeneMarketplace {
    modules: std::collections::HashMap<Uuid, GeneModule>,
    offers: Vec<GeneTransferOffer>,
}

impl GeneMarketplace {
    pub fn new() -> Self {
        Self::default()
    }

    /// List a gene module for sale.
    pub fn list_module(&mut self, module: GeneModule) {
        self.modules.insert(module.id, module);
    }

    /// Get a module by ID.
    pub fn get_module(&self, id: &Uuid) -> Option<&GeneModule> {
        self.modules.get(id)
    }

    /// Browse available modules sorted by fitness impact.
    pub fn browse(&self, limit: usize) -> Vec<&GeneModule> {
        let mut modules: Vec<&GeneModule> = self.modules.values().collect();
        modules.sort_by(|a, b| {
            b.avg_fitness_impact
                .partial_cmp(&a.avg_fitness_impact)
                .unwrap()
        });
        modules.truncate(limit);
        modules
    }

    /// Create a transfer offer.
    pub fn create_offer(
        &mut self,
        seller: AgentID,
        buyer: AgentID,
        module_id: Uuid,
    ) -> Result<GeneTransferOffer, EvolutionError> {
        let module = self
            .modules
            .get(&module_id)
            .ok_or_else(|| EvolutionError::IncompatibleTransfer("Module not found".into()))?;

        let offer = GeneTransferOffer::new(seller, buyer, module_id, module.price);
        self.offers.push(offer.clone());
        Ok(offer)
    }

    /// Complete a transfer (after ATP payment is verified externally).
    pub fn complete_offer(
        &mut self,
        offer_id: &Uuid,
        fitness_delta: f64,
    ) -> Result<&GeneModule, EvolutionError> {
        let offer = self
            .offers
            .iter_mut()
            .find(|o| o.id == *offer_id)
            .ok_or_else(|| EvolutionError::IncompatibleTransfer("Offer not found".into()))?;

        offer.complete();
        let module_id = offer.module_id;

        let module = self
            .modules
            .get_mut(&module_id)
            .ok_or_else(|| EvolutionError::IncompatibleTransfer("Module not found".into()))?;

        module.record_transfer(fitness_delta);
        Ok(module)
    }

    /// Total modules listed.
    pub fn module_count(&self) -> usize {
        self.modules.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn test_gene_module_creation() {
        let module = GeneModule::new(
            "optimizer-v1",
            "Optimizes compute loops",
            vec![1, 2, 3, 4],
            Uuid::new_v4(),
            10.0,
        )
        .unwrap();
        assert!(module.verify_integrity());
    }

    #[test]
    fn test_module_too_large() {
        let big_payload = vec![0u8; MAX_MODULE_SIZE + 1];
        assert!(GeneModule::new("big", "too big", big_payload, Uuid::new_v4(), 1.0).is_err());
    }

    #[test]
    fn test_marketplace_flow() {
        let mut market = GeneMarketplace::new();
        let creator = Uuid::new_v4();
        let buyer = Uuid::new_v4();

        let module = GeneModule::new(
            "sort-v2",
            "Efficient sorting",
            b"fn sort() {}".to_vec(),
            creator,
            5.0,
        )
        .unwrap();
        let module_id = module.id;

        market.list_module(module);
        assert_eq!(market.module_count(), 1);

        let offer = market.create_offer(creator, buyer, module_id).unwrap();
        market.complete_offer(&offer.id, 0.05).unwrap();

        let m = market.get_module(&module_id).unwrap();
        assert_eq!(m.transfer_count, 1);
    }
}
