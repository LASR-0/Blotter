//! Sync groundwork, shared with Blotter Sync (the home server).
//!
//! Everything a replica needs to take part in sync without any networking:
//! this device's identity and its hybrid logical clock so far. The sync
//! registry, change capture, the merge function and the wire types join
//! them (roadmap §14). Networking stays out of the core: the app and the
//! server each bring their own.

pub mod device;
pub mod hlc;

pub use device::Device;
pub use hlc::Hlc;
