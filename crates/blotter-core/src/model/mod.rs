//! Domain types. These serialize to the JSON the UI, AI tools and MCP see,
//! and map onto the SQLite schema in `migrations/`.

mod blockout;
mod change;
mod item;
mod reminder;
mod subscription;

pub use blockout::{Blockout, BlockoutException, Weekdays};
pub use change::{Actor, Change};
pub use item::{Anchoring, Item, ItemKind, ItemStatus, Source};
pub use reminder::Reminder;
pub use subscription::{Subscription, SubscriptionEvent};

/// Declares a fieldless enum stored as a lowercase string, with serde,
/// `as_str` and `FromStr` all agreeing on the same spelling.
macro_rules! string_enum {
    ($(#[$meta:meta])* $name:ident { $($variant:ident => $text:literal),+ $(,)? }) => {
        $(#[$meta])*
        #[derive(
            Debug, Clone, Copy, PartialEq, Eq, Hash,
            serde::Serialize, serde::Deserialize, schemars::JsonSchema,
        )]
        pub enum $name {
            $(#[serde(rename = $text)] $variant),+
        }

        impl $name {
            pub const ALL: &'static [$name] = &[$($name::$variant),+];

            pub fn as_str(self) -> &'static str {
                match self { $($name::$variant => $text),+ }
            }
        }

        impl std::str::FromStr for $name {
            type Err = $crate::error::CoreError;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s {
                    $($text => Ok($name::$variant),)+
                    other => Err($crate::error::CoreError::invalid(format!(
                        concat!("unknown ", stringify!($name), " '{}'"), other
                    ))),
                }
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str(self.as_str())
            }
        }
    };
}
pub(crate) use string_enum;
