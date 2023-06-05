pub mod basic {
    use crate::bindings::*;
    use crate::Locked;

    /// Blackboard Name Type
    pub type BlackboardName = ApexName;

    /// Blackboard Id Type
    ///
    /// According to ARINC 653P1-5 this may either be 32 or 64 bits.
    /// Internally we will use 64-bit by default.
    /// The implementing Hypervisor may cast this to 32-bit if needed
    pub type BlackboardId = ApexLongInteger;

    /// ARINC653P1-5 required functions for Blackboard functionality
    pub trait ApexBlackboardP1 {
        /// Creates new Blackboard
        #[cfg_attr(not(feature = "full_doc"), doc(hidden))]
        fn create_blackboard<L: Locked>(
            blackboard_name: BlackboardName,
            max_message_size: MessageSize,
        ) -> Result<BlackboardId, ErrorReturnCode>;

        /// Display specified message on Blackboard
        #[cfg_attr(not(feature = "full_doc"), doc(hidden))]
        fn display_blackboard<L: Locked>(
            blackboard_id: BlackboardId,
            message: &[ApexByte],
        ) -> Result<(), ErrorReturnCode>;

        /// Read message from Blackboard. If no message is available, this function waits as long as `timeout` specifies.
        ///
        /// # Safety
        ///
        /// This function is safe, as long as the buffer can hold whatever is read
        #[cfg_attr(not(feature = "full_doc"), doc(hidden))]
        unsafe fn read_blackboard<L: Locked>(
            blackboard_id: BlackboardId,
            time_out: ApexSystemTime,
            message: &mut [ApexByte],
        ) -> Result<MessageSize, ErrorReturnCode>;

        /// Clears the Blackboard content
        #[cfg_attr(not(feature = "full_doc"), doc(hidden))]
        fn clear_blackboard<L: Locked>(blackboard_id: BlackboardId) -> Result<(), ErrorReturnCode>;

        /// Get Blackboard Id from a given Name
        #[cfg_attr(not(feature = "full_doc"), doc(hidden))]
        fn get_blackboard_id<L: Locked>(
            blackboard_name: BlackboardName,
        ) -> Result<BlackboardId, ErrorReturnCode>;

        /// Get Blackboard Status
        #[cfg_attr(not(feature = "full_doc"), doc(hidden))]
        fn get_blackboard_status<L: Locked>(
            blackboard_id: BlackboardId,
        ) -> Result<BlackboardStatus, ErrorReturnCode>;
    }

    /// Enum representing whether Blackboard is empty or not
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

    /// Struct representing status information for a Blackboard
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

pub mod abstraction {
    use core::marker::PhantomData;
    use core::sync::atomic::AtomicPtr;

    // Reexport important basic-types for downstream-user
    pub use super::basic::{ApexBlackboardP1, BlackboardId, BlackboardStatus};
    use crate::bindings::*;
    use crate::hidden::Key;
    use crate::prelude::*;

    /// Blackboard Abstraction Struct
    #[derive(Debug, Clone)]
    pub struct Blackboard<B: ApexBlackboardP1> {
        _b: PhantomData<AtomicPtr<B>>,
        id: BlackboardId,
        size: MessageSize,
    }

    /// Free extra functions for implementer of [ApexBlackboardP1]
    pub trait ApexBlackboardP1Ext: ApexBlackboardP1 + Sized {
        /// Get a [Blackboard] by [Name]
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
                size: status.max_message_size,
            })
        }
    }

    impl<B: ApexBlackboardP1> Blackboard<B> {
        /// Get [Self] by [Name]
        pub fn from_name(name: Name) -> Result<Blackboard<B>, Error> {
            B::get_blackboard(name)
        }

        /// Return own Id
        pub fn id(&self) -> BlackboardId {
            self.id
        }

        /// Return MaxMessageSize of this Blackboard
        pub fn size(&self) -> MessageSize {
            self.size
        }

        /// Checked Blackboard write from specified buffer
        ///
        /// # Additional Errors:
        /// - WriteError:
        ///   - buffer got zero length
        ///   - buffer is larger than self.size()
        pub fn display(&self, buffer: &[ApexByte]) -> Result<(), Error> {
            WriteError::validate(self.size, buffer)?;
            B::display_blackboard::<Key>(self.id, buffer)?;
            Ok(())
        }

        /// Checked Blackboard read into specified buffer
        ///
        /// # Additional Errors:
        /// -
        pub fn read<'a>(
            &self,
            timeout: SystemTime,
            buffer: &'a mut [ApexByte],
        ) -> Result<&'a [ApexByte], Error> {
            ReadError::validate(self.size, buffer)?;
            unsafe { self.read_unchecked(timeout, buffer) }
        }

        /// Unchecked Blackboard read into specified buffer
        ///
        /// # Safety
        ///
        /// This function is safe, as long as the buffer can hold whatever is read
        pub unsafe fn read_unchecked<'a>(
            &self,
            timeout: SystemTime,
            buffer: &'a mut [ApexByte],
        ) -> Result<&'a [ApexByte], Error> {
            let len = B::read_blackboard::<Key>(self.id, timeout.into(), buffer)? as usize;
            Ok(&buffer[..len])
        }

        /// Clear Blackboard content
        pub fn clear(&self) {
            // According to ARINC653P1-5 3.7.2.2.4 this can only fail if the blackboard_id
            //  does not exist in the current partition.
            // But since we retrieve the blackboard_id directly from the hypervisor
            //  there is no possible way for it not existing
            B::clear_blackboard::<Key>(self.id).unwrap()
        }

        /// Get Status of this Blackboard
        pub fn status(&self) -> BlackboardStatus {
            // According to ARINC653P1-5 3.7.2.2.6 this can only fail if the blackboard_id
            //  does not exist in the current partition.
            // But since we retrieve the blackboard_id directly from the hypervisor
            //  there is no possible way for it not existing
            B::get_blackboard_status::<Key>(self.id).unwrap()
        }
    }

    impl<B: ApexBlackboardP1> StartContext<B> {
        /// Possible Errors:
        ///   - InvalidConfig
        ///     - Not enough available space
        ///     - Blackboard limit has been reached
        ///   - NoAction
        ///     - Blackboard with this name already exists
        ///   - InvalidParam:
        ///     - MessageSize is equal or less than zero
        pub fn create_blackboard(
            &mut self,
            name: Name,
            size: MessageSize,
        ) -> Result<Blackboard<B>, Error> {
            let id = B::create_blackboard::<Key>(name.into(), size)?;
            Ok(Blackboard {
                _b: Default::default(),
                id,
                size,
            })
        }
    }
}
