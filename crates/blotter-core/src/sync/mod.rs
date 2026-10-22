//! Sync groundwork, shared with Blotter Sync (the home server).
//!
//! Everything a replica needs to take part in sync without any networking:
//! this device's identity, its hybrid logical clock, and the registry of
//! what syncs. Change capture, the merge function and the wire types join
//! them (roadmap §14). Networking stays out of the core: the app and the
//! server each bring their own.

pub mod device;
pub mod hlc;
pub mod registry;

pub use device::Device;
pub use hlc::Hlc;
