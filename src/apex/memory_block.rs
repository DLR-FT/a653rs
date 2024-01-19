/// bindings for ARINC653P2-4 3.9 memory blocks
pub mod basic {
    use crate::apex::types::basic::*;

    /// ARINC653P2-4 3.9.1
    pub type MemoryBlockName = ApexName;
    /// ARINC653P2-4 3.9.1
    pub type MemoryBlockSize = ApexInteger;

    /// ARINC653P2-4 3.9.2 required functions for memory block functionality
    pub trait ApexMemoryBlockP2 {
        /// ARINC653P2-4 3.9.2.1
        ///
        /// # Errors
        /// - [ErrorReturnCode::InvalidConfig]: memory block with `memory_block_name` does not exist
        #[cfg_attr(not(feature = "full_doc"), doc(hidden))]
        fn get_memory_block_status(
            memory_block_name: MemoryBlockName,
        ) -> Result<ApexMemoryBlockStatus, ErrorReturnCode>;
    }

    /// ARINC653P2-4 3.9.1
    #[repr(u32)]
    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
    #[cfg_attr(feature = "strum", derive(strum::FromRepr))]
    pub enum MemoryBlockMode {
        Read = 0,
        ReadWrite = 1,
    }

    /// ARINC653P2-4
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct ApexMemoryBlockStatus {
        /// pointer to memory block start
        pub address: *mut ApexByte,
        /// memory block length in bytes
        pub size: MemoryBlockSize,
        pub mode: MemoryBlockMode,
    }
}

/// abstraction for ARINC653P2-4 3.9 memory blocks
pub mod abstraction {
    use core::marker::PhantomData;
    use core::slice::from_raw_parts_mut;

    use super::basic::{ApexMemoryBlockP2, ApexMemoryBlockStatus};
    // Reexport important basic-types for downstream-user
    pub use super::basic::{MemoryBlockMode, MemoryBlockName, MemoryBlockSize};
    use crate::prelude::*;

    // #[derive(Debug)]
    // pub struct MemoryBlock<M: ApexMemoryBlockP2> {
    //     _b: PhantomData<AtomicPtr<M>>,
    //     address: MemoryBlockAddress,
    //     size:
    // }
    /// Memory block abstraction struct
    #[derive(Debug, PartialEq, Eq)]
    pub struct MemoryBlock<M: ApexMemoryBlockP2> {
        _m: PhantomData<M>,
        memory: &'static mut [ApexByte],
        mode: MemoryBlockMode,
    }

    impl<M: ApexMemoryBlockP2> From<ApexMemoryBlockStatus> for MemoryBlock<M> {
        fn from(m: ApexMemoryBlockStatus) -> Self {
            let memory = unsafe { from_raw_parts_mut(m.address, m.size as usize) };
            MemoryBlock {
                _m: Default::default(),
                memory,
                mode: m.mode,
            }
        }
    }

    /// Free extra functions for implementer of [ApexMemoryBlockP2]
    pub trait ApexMemoryBlockP2Ext: ApexMemoryBlockP2 + Sized {
        /// # Errors
        /// - [Error::InvalidConfig]: memory block with `name` does not exist
        fn get_memory_block(name: Name) -> Result<MemoryBlock<Self>, Error>;
    }

    impl<M: ApexMemoryBlockP2> ApexMemoryBlockP2Ext for M {
        fn get_memory_block(name: Name) -> Result<MemoryBlock<M>, Error> {
            Ok(M::get_memory_block_status(name.into())?.into())
        }
    }

    impl<M: ApexMemoryBlockP2> MemoryBlock<M> {
        /// # Errors
        /// - [Error::InvalidConfig]: memory block with `name` does not exist
        pub fn from_name(name: Name) -> Result<MemoryBlock<M>, Error> {
            M::get_memory_block(name)
        }

        /// # Safety
        /// Because this memory block can change externally, no borrow checker guarantees can be given
        pub unsafe fn read(&self) -> &[ApexByte] {
            self.memory
        }

        /// # Options
        /// - [Some]: access mode is [MemoryBlockMode::ReadWrite]
        /// - [None]: access mode is [MemoryBlockMode::Read]
        ///
        /// # Safety
        /// Because this memory block can change externally, no borrow checker guarantees can be given.  
        /// Further, multiple [MemoryBlock] structs can be requested, rendering partition internal mutable borrow checking useless aswell.
        pub unsafe fn write(&mut self) -> Option<&mut [ApexByte]> {
            if self.mode == MemoryBlockMode::ReadWrite {
                return Some(self.memory);
            }
            None
        }

        pub fn mode(&self) -> MemoryBlockMode {
            self.mode
        }
    }
}
