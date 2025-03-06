use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

use crate::hardcoded::DNA;

#[derive(Serialize, Deserialize)]
pub struct SporeData {
    pub content_type: Vec<u8>,
    pub content: Vec<u8>,
    pub cluster_id: Option<Vec<u8>>,
}

impl SporeData {
    pub fn dna(&self) -> Option<DNA> {
        // Try to treat content as JSON format
        if let Ok(value) = serde_json::from_slice::<serde_json::Value>(&self.content) {
            let dna = match &value {
                serde_json::Value::String(_) => &value,
                serde_json::Value::Array(array) => array.first()?,
                serde_json::Value::Object(object) => object.get("dna")?,
                _ => return None,
            };

            if let serde_json::Value::String(string) = dna {
                if let Ok(decoded) = hex::decode(string) {
                    return decoded.try_into().ok();
                }
            }
            return None;
        }
        // Otherwise, treat content as plain HEX string
        let content = core::str::from_utf8(&self.content).ok()?;
        let decoded = hex::decode(content).ok()?;
        decoded.try_into().ok()
    }
}
