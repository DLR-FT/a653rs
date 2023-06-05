pub mod basic {
    use crate::bindings::*;

    pub type MemoryBlockName = ApexName;
    pub type MemoryBlockSize = ApexInteger;
    pub type MemoryBlockAddress = *mut u8;

    pub trait ApexMemoryBlockP2 {
        #[cfg_attr(not(feature = "full_doc"), doc(hidden))]
        fn get_memory_block_status<L: Locked>(
            memory_block_name: MemoryBlockName,
        ) -> Result<ApexMemoryBlockStatus, ErrorReturnCode>;
    }

    #[repr(u32)]
    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
    #[cfg_attr(feature = "strum", derive(strum::FromRepr))]
    pub enum MemoryBlockMode {
        Read = 0,
        ReadWrite = 1,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct ApexMemoryBlockStatus {
        pub address: MemoryBlockAddress,
        pub size: MemoryBlockSize,
        pub mode: MemoryBlockMode,
    }
}
// TODO continue with Wanja here
pub mod abstraction {
    // use core::marker::PhantomData;

    // use super::basic::MemoryBlockAddress;
    pub use super::basic::{ApexMemoryBlockP2, MemoryBlockMode, MemoryBlockName, MemoryBlockSize};
    // use crate::hidden::Key;
    // use crate::prelude::*;

    // #[derive(Debug)]
    // pub struct MemoryBlock<M: ApexMemoryBlockP2> {
    //     _b: PhantomData<AtomicPtr<M>>,
    //     address: MemoryBlockAddress,
    //     size:
    // }
}
