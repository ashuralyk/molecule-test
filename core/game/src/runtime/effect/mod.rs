use core::cell::RefCell;

use alloc::{boxed::Box, vec, vec::Vec};
use common::effect::{Effect, EffectName};
use serde::{Deserialize, Serialize};

use crate::{Context, Error, Signal};

mod feature;

macro_rules! impls {
    ($var:ident, [$($name:ident,)+]) => {
        match $var {
            $(EffectName::$name => Box::<feature::$name>::default(),)+
        }
    };
}

pub fn _default_run(_: &Signal, _: &EffectRuntime) -> Result<Vec<Signal>, Error> {
    Ok(vec![])
}

pub fn default_transform(
    _: Vec<Signal>,
    _: &Signal,
    _: &EffectRuntime,
    _: &mut Context,
) -> Result<Vec<Signal>, Error> {
    Ok(vec![])
}

#[allow(unused)]
pub trait Feature: Send + Sync {
    fn run(
        &mut self,
        signal: &Signal,
        effect: &EffectRuntime,
        _: &mut Context,
    ) -> Result<Vec<Signal>, Error> {
        _default_run(signal, effect)
    }

    fn transform(
        &self,
        signal: &Signal,
        effect: &EffectRuntime,
        ctx: &mut Context,
    ) -> Result<Vec<Signal>, Error> {
        default_transform(vec![], signal, effect, ctx)
    }
}

#[derive(Serialize)]
pub struct EffectRuntime {
    pub raw: Effect,
    pub runtime_id: u16,
    pub parent_runtime_id: u16,
    pub value: u8,
    pub countdown: u8,
    #[serde(skip)]
    pub _feature: RefCell<Box<dyn Feature>>,
}

impl<'de> Deserialize<'de> for EffectRuntime {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct EffectRuntimeHelper {
            raw: Effect,
            runtime_id: u16,
            parent_runtime_id: u16,
            value: u8,
            countdown: u8,
        }
        let helper = EffectRuntimeHelper::deserialize(deserializer)?;
        let feature = RefCell::new(EffectFactory::create_feature(&helper.raw.name));
        Ok(EffectRuntime {
            raw: helper.raw,
            runtime_id: helper.runtime_id,
            parent_runtime_id: helper.parent_runtime_id,
            value: helper.value,
            countdown: helper.countdown,
            _feature: feature,
        })
    }
}

#[cfg(feature = "debug")]
impl core::fmt::Debug for EffectRuntime {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("EffectRuntime")
            .field("raw", &self.raw)
            .field("runtime_id", &self.runtime_id)
            .field("parent_runtime_id", &self.parent_runtime_id)
            .field("value", &self.value)
            .field("countdown", &self.countdown)
            .field("feature", &"<dyn Feature>")
            .finish()
    }
}

impl EffectRuntime {
    pub fn run(&mut self, _signal: &Signal, _ctx: &mut Context) -> Result<(), Error> {
        Ok(())
    }

    pub fn transform(&self, _signal: &Signal, _ctx: &mut Context) -> Result<Vec<Signal>, Error> {
        Ok(vec![])
    }
}

pub struct EffectFactory {}

impl EffectFactory {
    fn create_feature(effect_name: &EffectName) -> Box<dyn Feature> {
        impls!(effect_name, [ExtraDamage,])
    }
}
