use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub enum ValueType {
    #[serde(rename = "random")]
    Random(u16, u16),
    #[serde(rename = "fixed")]
    Fixed(u16),
}

impl ValueType {
    pub fn value_u8(&self, seed: u8) -> u8 {
        self.value_u16(seed) as u8
    }

    pub fn value_u16(&self, seed: u8) -> u16 {
        match self {
            ValueType::Random(min, max) => seed as u16 % (max - min) + min,
            ValueType::Fixed(value) => *value,
        }
    }
}
