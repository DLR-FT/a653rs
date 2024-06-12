//! Abstraction layer for the ARINC653 P1/P2/P4 API

#![no_std]
#![warn(clippy::missing_errors_doc)]
#![warn(clippy::missing_panics_doc)]
#![warn(rustdoc::missing_crate_level_docs)]
#![deny(rustdoc::broken_intra_doc_links)]

/// Bindings to traits which are supposed to be implemented for ARINC653 compliant hypervisors
#[cfg(feature = "bindings")]
pub mod bindings;

/// Standard prelude to be used by application software and high-level drivers
pub mod prelude;

pub(crate) mod apex;

#[cfg(feature = "macros")]
pub use a653rs_macros::*;
