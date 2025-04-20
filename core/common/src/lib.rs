#![no_std]
extern crate alloc;

pub mod hardcoded;
pub mod value;

#[cfg(feature = "card")]
pub mod card;

#[cfg(feature = "contract")]
pub mod contract;

#[cfg(feature = "effect")]
pub mod effect;

#[cfg(feature = "enemy")]
pub mod enemy;

#[cfg(feature = "operation")]
pub mod operation;

#[cfg(feature = "player")]
pub mod player;

#[macro_export]
macro_rules! enum_with_display {
    (
        $(#[derive($($feature:ident$(,)?)+)])?
        $(#[cfg_attr($meta:meta, $attr:meta)])*
        pub enum $struct:ident {$($member:ident$(,)?)+}
    ) => {
        $(#[derive($($feature,)+)])?
        $(#[cfg_attr($meta, $attr)])*
        pub enum $struct {
            $($member,)+
        }

        impl core::fmt::Display for $struct {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                match self {
                    $($struct::$member => write!(f, "{}", stringify!($member)),)+
                }
            }
        }
    };
}
