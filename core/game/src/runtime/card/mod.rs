use alloc::{boxed::Box, format, vec::Vec};
use common::card::{get_card_template_by_name, instance_card_by_seed, Card, CardName};
use core::cell::RefCell;
use database::CARD_POOL;
use serde::{Deserialize, Serialize};

use crate::{err, Context, Error, Signal};

macro_rules! impls {
    ($var:ident, [$($name:ident,)+]) => {
        match $var {
            $(CardName::$name => Box::<implementation::$name>::default(),)+
        }
    };
}

pub mod implementation;

#[allow(unused)]
pub trait Implementation: Send + Sync {
    fn run(
        &mut self,
        signal: &Signal,
        card: &CardRuntime,
        ctx: &mut Context,
    ) -> Result<Vec<Signal>, Error>;
}

pub struct CardRuntime {
    pub raw: Card,
    pub parent_runtime_id: u16,
    pub runtime_id: u16,
    pub cost: u8,
    pub exile: bool,
    pub awake: u8,
    pub _implementation: RefCell<Box<dyn Implementation>>,
}

impl Serialize for CardRuntime {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        #[derive(Serialize)]
        struct CardRuntimeHelper {
            seed: u64,
            card_name: CardName,
            parent_runtime_id: u16,
            runtime_id: u16,
            cost: u8,
            exile: bool,
            awake: u8,
        }
        let helper = CardRuntimeHelper {
            seed: self.raw.seed,
            card_name: self.raw.name,
            parent_runtime_id: self.parent_runtime_id,
            runtime_id: self.runtime_id,
            cost: self.cost,
            exile: self.exile,
            awake: self.awake,
        };
        helper.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for CardRuntime {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct CardRuntimeHelper {
            seed: u64,
            card_name: CardName,
            parent_runtime_id: u16,
            runtime_id: u16,
            cost: u8,
            exile: bool,
            awake: u8,
        }
        let helper = CardRuntimeHelper::deserialize(deserializer)?;
        let implementation = RefCell::new(CardFactory::create_implementation(&helper.card_name));
        let template = get_card_template_by_name(CARD_POOL, helper.card_name).ok_or_else(|| {
            serde::de::Error::custom(format!("card template not found: {}", helper.card_name))
        })?;
        Ok(CardRuntime {
            raw: instance_card_by_seed(&template, helper.seed),
            parent_runtime_id: helper.parent_runtime_id,
            runtime_id: helper.runtime_id,
            cost: helper.cost,
            exile: helper.exile,
            awake: helper.awake,
            _implementation: implementation,
        })
    }
}

#[cfg(feature = "debug")]
impl core::fmt::Debug for CardRuntime {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("CardRuntime")
            .field("raw", &self.raw)
            .field("parent_runtime_id", &self.parent_runtime_id)
            .field("runtime_id", &self.runtime_id)
            .field("cost", &self.cost)
            .field("exile", &self.exile)
            .field("awake", &self.awake)
            .field("implementation", &"<implementation>")
            .finish()
    }
}

impl CardRuntime {
    pub fn run(&mut self, _signal: &Signal, _ctx: &mut Context) -> Result<(), Error> {
        Ok(())
    }

    pub fn param_value(&self, offset: usize) -> Result<u16, Error> {
        match offset {
            0 => Ok(self.raw.value_0 as u16),
            1 => Ok(self.raw.value_1 as u16),
            _ => Err(err!(CardInvalidConfig(self.raw.name, offset))),
        }
    }
}

pub struct CardFactory {}

impl CardFactory {
    fn create_implementation(card_name: &CardName) -> Box<dyn Implementation> {
        impls!(card_name, [Attack,])
    }
}
