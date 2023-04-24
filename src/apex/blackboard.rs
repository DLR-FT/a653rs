/// bindings for ARINC653P1-5 3.7.2.2 blackboard
pub mod basic {
    use crate::bindings::*;
    use crate::Locked;

    /// ARINC653P1-5 3.7.1
    pub type BlackboardName = ApexName;

    /// ARINC653P1-5 3.7.1
    ///
    /// According to ARINC 653P1-5 this may either be 32 or 64 bits.
    /// Internally we will use 64-bit by default.
    /// The implementing hypervisor may cast this to 32-bit if needed
    pub type BlackboardId = ApexLongInteger;

    /// ARINC653P1-5 3.7.2.2 required functions for blackboard functionality
    pub trait ApexBlackboardP1 {
        /// APEX653P1-5 3.7.2.2.1
        ///
        /// # Errors
        /// - [ErrorReturnCode::InvalidConfig]: not enough memory is available
        /// - [ErrorReturnCode::InvalidConfig]: [ApexLimits::SYSTEM_LIMIT_NUMBER_OF_BLACKBOARDS](crate::bindings::ApexLimits::SYSTEM_LIMIT_NUMBER_OF_BLACKBOARDS) was reached
        /// - [ErrorReturnCode::NoAction]: a blackboard with given `blackboard_name` already exists
        /// - [ErrorReturnCode::InvalidParam]: `max_message_size` is zero
        /// - [ErrorReturnCode::InvalidMode]: our current operating mode is [OperatingMode::Normal](crate::prelude::OperatingMode::Normal)
        #[cfg_attr(not(feature = "full_doc"), doc(hidden))]
        fn create_blackboard<L: Locked>(
            blackboard_name: BlackboardName,
            max_message_size: MessageSize,
        ) -> Result<BlackboardId, ErrorReturnCode>;

        /// APEX653P1-5 3.7.2.2.2
        ///
        /// # Errors
        /// - [ErrorReturnCode::InvalidParam]: blackboard with `blackboard_id` does not exist
        /// - [ErrorReturnCode::InvalidParam]: the `message` is longer than the `max_message_size` specified for this blackboard
        /// - [ErrorReturnCode::InvalidParam]: `message` length is zero
        #[cfg_attr(not(feature = "full_doc"), doc(hidden))]
        fn display_blackboard<L: Locked>(
            blackboard_id: BlackboardId,
            message: &[ApexByte],
        ) -> Result<(), ErrorReturnCode>;

        /// APEX653P1-5 3.7.2.2.3
        ///
        /// # Errors
        /// - [ErrorReturnCode::InvalidParam]: blackboard with `blackboard_id` does not exist
        /// - [ErrorReturnCode::InvalidParam]: `time_out` is invalid
        /// - [ErrorReturnCode::InvalidMode]: current process holds a mutex
        /// - [ErrorReturnCode::InvalidMode]: current process is error handler AND `time_out` is not instant.
        /// - [ErrorReturnCode::NotAvailable]: there is no message on the blackboard
        /// - [ErrorReturnCode::TimedOut]: `time_out` elapsed
        ///
        /// # Safety
        ///
        /// This function is safe, as long as the `message` can hold whatever is read
        #[cfg_attr(not(feature = "full_doc"), doc(hidden))]
        unsafe fn read_blackboard<L: Locked>(
            blackboard_id: BlackboardId,
            time_out: ApexSystemTime,
            message: &mut [ApexByte],
        ) -> Result<MessageSize, ErrorReturnCode>;

        /// APEX653P1-5 3.7.2.2.4
        ///
        /// # Errors
        /// - [ErrorReturnCode::InvalidParam]: blackboard with `blackboard_id` does not exist
        #[cfg_attr(not(feature = "full_doc"), doc(hidden))]
        fn clear_blackboard<L: Locked>(blackboard_id: BlackboardId) -> Result<(), ErrorReturnCode>;

        /// APEX653P1-5 3.7.2.2.5
        ///
        /// # Errors
        /// - [ErrorReturnCode::InvalidConfig]: blackboard with `blackboard_name` does not exist
        #[cfg_attr(not(feature = "full_doc"), doc(hidden))]
        fn get_blackboard_id<L: Locked>(
            blackboard_name: BlackboardName,
        ) -> Result<BlackboardId, ErrorReturnCode>;

        /// APEX653P1-5 3.7.2.2.6
        ///
        /// # Errors
        /// - [ErrorReturnCode::InvalidParam]: blackboard with `blackboard_id` does not exist
        #[cfg_attr(not(feature = "full_doc"), doc(hidden))]
        fn get_blackboard_status<L: Locked>(
            blackboard_id: BlackboardId,
        ) -> Result<BlackboardStatus, ErrorReturnCode>;
    }

    /// ARINC653P1-5 3.7.1
    #[repr(u32)]
    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
    #[cfg_attr(feature = "strum", derive(strum::FromRepr))]
    pub enum EmptyIndicator {
        /// Indicator that Blackboard is empty
        Empty = 0,
        /// Indicator that Blackboard is not empty
        Occupied = 1,
    }

    /// ARINC653P1-5 3.7.1
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct BlackboardStatus {
        /// If the Blackboard is empty or not
        pub empty_indicator: EmptyIndicator,
        /// How long the message on the Blackboard may be
        pub max_message_size: MessageSize,
        /// how many processes are currently waiting on a blackboard value
        pub waiting_processes: WaitingRange,
    }
}

/// abstraction for ARINC653P1-5 3.7.2.2 blackboard
pub mod abstraction {
    use core::marker::PhantomData;
    use core::sync::atomic::AtomicPtr;

    // Reexport important basic-types for downstream-user
    pub use super::basic::{ApexBlackboardP1, BlackboardId, BlackboardStatus};
    use crate::bindings::*;
    use crate::hidden::Key;
    use crate::prelude::*;

    /// Blackboard abstraction struct
    #[derive(Debug, Clone)]
    pub struct Blackboard<B: ApexBlackboardP1> {
        _b: PhantomData<AtomicPtr<B>>,
        id: BlackboardId,
        max_size: MessageSize,
    }

    /// Free extra functions for implementer of [ApexBlackboardP1]
    pub trait ApexBlackboardP1Ext: ApexBlackboardP1 + Sized {
        /// # Errors
        /// - [Error::InvalidConfig]: blackboard with `name` does not exist
        fn get_blackboard(name: Name) -> Result<Blackboard<Self>, Error>;
    }

    impl<B: ApexBlackboardP1> ApexBlackboardP1Ext for B {
        fn get_blackboard(name: Name) -> Result<Blackboard<B>, Error> {
            let id = B::get_blackboard_id::<Key>(name.into())?;
            // According to ARINC653P1-5 3.7.2.2.6 this can only fail if the blackboard_id
            //  does not exist in the current partition.
            // But since we retrieve the blackboard_id directly from the hypervisor
            //  there is no possible way for it not existing
            let status = B::get_blackboard_status::<Key>(id).unwrap();

            Ok(Blackboard {
                _b: Default::default(),
                id,
                max_size: status.max_message_size,
            })
        }
    }

    impl<B: ApexBlackboardP1> Blackboard<B> {
        /// # Errors
        /// - [Error::InvalidConfig]: blackboard with `name` does not exist
        pub fn from_name(name: Name) -> Result<Blackboard<B>, Error> {
            B::get_blackboard(name)
        }

        pub fn id(&self) -> BlackboardId {
            self.id
        }

        pub fn size(&self) -> MessageSize {
            self.max_size
        }

        /// Checked blackboard write from specified buffer
        ///
        /// # Errors
        /// - [Error::WriteError]: the `buffer` is longer than the `max_message_size` specified for this blackboard
        /// - [Error::WriteError]: `buffer` length is zero
        pub fn display(&self, buffer: &[ApexByte]) -> Result<(), Error> {
            buffer.validate_write(self.max_size)?;
            B::display_blackboard::<Key>(self.id, buffer)?;
            Ok(())
        }

        /// Checked Blackboard read into specified buffer
        ///
        /// # Errors
        /// - [Error::InvalidParam]: `timeout` is invalid
        /// - [Error::InvalidMode]: current process holds a mutex
        /// - [Error::InvalidMode]: current process is error handler AND `timeout` is not instant.
        /// - [Error::NotAvailable]: there is no message on the blackboard
        /// - [Error::TimedOut]: `timeout` elapsed
        /// - [Error::ReadError]: prodived `buffer` is too small for this [Blackboard]'s `max_message_size`
        pub fn read<'a>(
            &self,
            timeout: SystemTime,
            buffer: &'a mut [ApexByte],
        ) -> Result<&'a [ApexByte], Error> {
            buffer.validate_read(self.max_size)?;
            unsafe { self.read_unchecked(timeout, buffer) }
        }

        /// Unchecked Blackboard read into specified buffer
        ///
        /// # Errors
        /// - [Error::InvalidParam]: `timeout` is invalid
        /// - [Error::InvalidMode]: current process holds a mutex
        /// - [Error::InvalidMode]: current process is error handler AND `timeout` is not instant.
        /// - [Error::NotAvailable]: there is no message on the blackboard
        /// - [Error::TimedOut]: `timeout` elapsed
        ///
        /// # Safety
        ///
        /// This function is safe, as long as the `buffer` can hold whatever is read
        pub unsafe fn read_unchecked<'a>(
            &self,
            timeout: SystemTime,
            buffer: &'a mut [ApexByte],
        ) -> Result<&'a [ApexByte], Error> {
            let len = B::read_blackboard::<Key>(self.id, timeout.into(), buffer)? as usize;
            Ok(&buffer[..len])
        }

        /// # Panics
        /// if this blackboard does not exist anymore
        pub fn clear(&self) {
            // According to ARINC653P1-5 3.7.2.2.4 this can only fail if the blackboard_id
            //  does not exist in the current partition.
            // But since we retrieve the blackboard_id directly from the hypervisor
            //  there is no possible way for it not existing
            B::clear_blackboard::<Key>(self.id).unwrap()
        }

        /// # Panics
        /// if this blackboard does not exist anymore
        pub fn status(&self) -> BlackboardStatus {
            // According to ARINC653P1-5 3.7.2.2.6 this can only fail if the blackboard_id
            //  does not exist in the current partition.
            // But since we retrieve the blackboard_id directly from the hypervisor
            //  there is no possible way for it not existing
            B::get_blackboard_status::<Key>(self.id).unwrap()
        }
    }

    impl<B: ApexBlackboardP1> StartContext<B> {
        /// # Errors
        /// - [Error::InvalidConfig]: not enough memory is available
        /// - [Error::InvalidConfig]: [ApexLimits::SYSTEM_LIMIT_NUMBER_OF_BLACKBOARDS](crate::bindings::ApexLimits::SYSTEM_LIMIT_NUMBER_OF_BLACKBOARDS) was reached
        /// - [Error::NoAction]: a blackboard with given `name` already exists
        /// - [Error::InvalidParam]: `size` is zero
        pub fn create_blackboard(
            &mut self,
            name: Name,
            size: MessageSize,
        ) -> Result<Blackboard<B>, Error> {
            let id = B::create_blackboard::<Key>(name.into(), size)?;
            Ok(Blackboard {
                _b: Default::default(),
                id,
                max_size: size,
            })
        }
    }
}
