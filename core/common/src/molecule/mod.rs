mod dob;
mod operation;

pub use dob::*;
pub use operation::*;

#[macro_export]
macro_rules! enum_with_display {
    (#[derive($($feature:ident$(,)?)+)] pub enum $struct:ident {$($member:ident$(,)?)+}) => {
        #[derive($($feature,)+)]
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
