use alloc::vec::Vec;
use ckb_std::{
    ckb_constants::Source,
    ckb_types::packed,
    error::SysError,
    high_level::{load_cell_capacity, load_cell_data, load_cell_lock, load_cell_type, QueryIter},
};
use serde::{Deserialize, Serialize};

use crate::hardcoded;

#[derive(Serialize, Deserialize)]
pub struct SporeData {
    pub content_type: Vec<u8>,
    pub content: Vec<u8>,
    pub cluster_id: Option<Vec<u8>>,
}

impl SporeData {
    pub fn dna(&self) -> Option<hardcoded::DNA> {
        let content = core::str::from_utf8(&self.content).ok()?;
        let decoded = hex::decode(content).ok()?;
        decoded.try_into().ok()
    }
}

#[derive(Serialize, Deserialize, PartialEq, Default)]
pub struct Script {
    pub code_hash: [u8; 32],
    pub hash_type: u8,
    pub args: Vec<u8>,
}

impl PartialEq<Script> for ckb_std::ckb_types::packed::Script {
    fn eq(&self, other: &Script) -> bool {
        self.code_hash().raw_data().as_ref() == other.code_hash
            && self.hash_type() == other.hash_type.into()
            && self.args().raw_data().as_ref() == other.args
    }
}

#[derive(Serialize, Deserialize, Default)]
pub struct DobGlobalStatistics {
    pub protocol_payee: Script,
    pub protocol_owner_hash: [u8; 32],
    pub blindbox_cluster_id: [u8; 32],
    pub card_cluster_id: [u8; 32],
    pub global_unboxed_count: u32,
    pub ckb_base: u64,
    pub ckb_increase_per_unbox: u64,
}

impl DobGlobalStatistics {
    fn search_spores(
        &self,
        source: Source,
        cluster_id: &[u8; 32],
    ) -> Result<Vec<(usize, packed::Script, SporeData)>, SysError> {
        let mut spores = Vec::new();
        for (i, type_) in QueryIter::new(load_cell_type, source).enumerate() {
            let Some(type_) = type_ else {
                continue;
            };
            let code_hash = type_.code_hash().raw_data();
            if hardcoded::SPORE_CODE_HASH_SET
                .iter()
                .all(|hash| hash != code_hash.as_ref())
            {
                continue;
            }
            let data = load_cell_data(i, source)?;
            let spore_data: SporeData =
                serde_molecule::from_slice(&data, false).map_err(|_| SysError::Encoding)?;
            if spore_data.cluster_id.as_ref().map(AsRef::as_ref) == Some(cluster_id.as_ref()) {
                spores.push((i, type_, spore_data));
            }
        }
        Ok(spores)
    }

    pub fn search_blindbox_dobs(
        &self,
        source: Source,
    ) -> Result<Vec<(usize, packed::Script, SporeData)>, SysError> {
        self.search_spores(source, &self.blindbox_cluster_id)
    }

    pub fn search_card_dobs(
        &self,
        source: Source,
    ) -> Result<Vec<(usize, packed::Script, SporeData)>, SysError> {
        self.search_spores(source, &self.card_cluster_id)
    }

    pub fn get_ckb_payment(&self) -> u64 {
        QueryIter::new(load_cell_lock, Source::Output)
            .enumerate()
            .filter_map(|(i, lock)| {
                if lock == self.protocol_payee {
                    let ckb_capacity = load_cell_capacity(i, Source::Output).unwrap_or_default();
                    Some(ckb_capacity)
                } else {
                    None
                }
            })
            .sum::<u64>()
    }
}

#[derive(Serialize, Deserialize)]
pub struct PveSessionMaterials {
    pub dna_collection: Vec<hardcoded::DNA>,
    pub archive_input: Vec<u8>,
    pub archive_output: Vec<u8>,
}
