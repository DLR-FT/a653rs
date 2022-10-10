//! Abstraction Layer for the ARINC653 P1/P2/P4 API

#![no_std]
// #![warn(missing_docs)]
#![deny(rustdoc::broken_intra_doc_links)]

/// Bindings to Traits which are supposed to be implemented for ARINC653 compliant Hypervisors
pub mod bindings;
/// Standard Prelude to be used by Application Software and High-level drivers
pub mod prelude;

mod apex;

//TODO pub use macros

// replace all <an APEX integer type> with i64

pub(crate) mod hidden {
    pub enum Key {}

    pub trait IsHidden {}

    impl crate::Locked for Key {}
    impl IsHidden for Key {}
}

/// Trait, designed for restricting access to [bindings] functions
pub trait Locked: hidden::IsHidden {}
