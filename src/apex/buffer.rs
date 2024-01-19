/// bindings for ARINC653P1-5 3.7.2.1 buffer
pub mod basic {
    use crate::apex::time::basic::*;
    use crate::apex::types::basic::*;

    /// ARINC653P1-5 3.7.1
    pub type BufferName = ApexName;

    /// ARINC653P1-5 3.7.1
    ///
    /// According to ARINC 653P1-5 this may either be 32 or 64 bits.
    /// Internally we will use 64-bit by default.
    /// The implementing Hypervisor may cast this to 32-bit if needed
    pub type BufferId = ApexLongInteger;

    /// ARINC653P1-5 3.7.2.1 required functions for buffer functionality
    pub trait ApexBufferP1 {
        /// APEX653P1-5 3.7.2.1.1
        ///
        /// # Errors
        /// - [ErrorReturnCode::InvalidConfig]: not enough memory is available
        /// - [ErrorReturnCode::InvalidConfig]: [ApexLimits::SYSTEM_LIMIT_NUMBER_OF_BUFFERS](crate::apex::limits::ApexLimits::SYSTEM_LIMIT_NUMBER_OF_BUFFERS) was reached
        /// - [ErrorReturnCode::NoAction]: a buffer with given `buffer_name` already exists
        /// - [ErrorReturnCode::InvalidParam]: `max_message_size` is zero
        /// - [ErrorReturnCode::InvalidParam]: `max_nb_message` is too large
        /// - [ErrorReturnCode::InvalidMode]: our current operating mode is [OperatingMode::Normal](crate::prelude::OperatingMode::Normal)
        fn create_buffer(
            buffer_name: BufferName,
            max_message_size: MessageSize,
            max_nb_message: MessageRange,
            queuing_discipline: QueuingDiscipline,
        ) -> Result<BufferId, ErrorReturnCode>;

        /// APEX653P1-5 3.7.2.1.2
        ///
        /// # Errors
        /// - [ErrorReturnCode::InvalidParam]: buffer with `buffer_id` does not exist
        /// - [ErrorReturnCode::InvalidParam]: `time_out` is invalid
        /// - [ErrorReturnCode::InvalidMode]: current process holds a mutex
        /// - [ErrorReturnCode::InvalidMode]: current process is error handler AND `time_out` is not instant.
        /// - [ErrorReturnCode::NotAvailable]: there is no place in the buffer
        /// - [ErrorReturnCode::TimedOut]: `time_out` elapsed
        fn send_buffer(
            buffer_id: BufferId,
            message: &[ApexByte],
            time_out: ApexSystemTime,
        ) -> Result<(), ErrorReturnCode>;

        /// APEX653P1-5 3.7.2.1.3
        ///
        /// # Errors
        /// - [ErrorReturnCode::InvalidParam]: buffer with `buffer_id` does not exist
        /// - [ErrorReturnCode::InvalidParam]: `time_out` is invalid
        /// - [ErrorReturnCode::InvalidMode]: current process holds a mutex
        /// - [ErrorReturnCode::InvalidMode]: current process is error handler AND `time_out` is not instant.
        /// - [ErrorReturnCode::NotAvailable]: there is no message in the buffer
        /// - [ErrorReturnCode::TimedOut]: `time_out` elapsed
        ///
        /// # Safety
        ///
        /// This function is safe, as long as the `message` can hold whatever is received
        unsafe fn receive_buffer(
            buffer_id: BufferId,
            time_out: ApexSystemTime,
            message: &mut [ApexByte],
        ) -> Result<MessageSize, ErrorReturnCode>;

        /// APEX653P1-5 3.7.2.1.4
        ///
        /// # Errors
        /// - [ErrorReturnCode::InvalidConfig]: buffer with `buffer_name` does not exist
        fn get_buffer_id(buffer_name: BufferName) -> Result<BufferId, ErrorReturnCode>;

        /// APEX653P1-5 3.7.2.1.5
        ///
        /// # Errors
        /// - [ErrorReturnCode::InvalidParam]: buffer with `buffer_id` does not exist
        fn get_buffer_status(buffer_id: BufferId) -> Result<BufferStatus, ErrorReturnCode>;
    }

    /// ARINC653P1-5 3.7.1
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct BufferStatus {
        /// number of messages in the buffer
        pub nb_message: MessageRange,
        /// maximum number of messages which can fit in this buffer
        pub max_nb_message: MessageRange,
        pub max_message_size: MessageSize,
        pub waiting_processes: WaitingRange,
    }
}

/// abstraction for ARINC653P1-5 3.7.2.1 blackboard
pub mod abstraction {
    use core::marker::PhantomData;
    use core::sync::atomic::AtomicPtr;

    use super::basic::ApexBufferP1;
    // Reexport important basic-types for downstream-user
    pub use super::basic::{BufferId, BufferStatus};
    use crate::prelude::*;

    /// Buffer abstraction struct
    #[derive(Debug)]
    pub struct Buffer<B: ApexBufferP1> {
        _b: PhantomData<AtomicPtr<B>>,
        id: BufferId,
        max_size: MessageSize,
        max_number_msgs: MessageRange,
    }

    impl<B: ApexBufferP1> Clone for Buffer<B> {
        fn clone(&self) -> Self {
            Self {
                _b: self._b,
                id: self.id,
                max_size: self.max_size,
                max_number_msgs: self.max_number_msgs,
            }
        }
    }

    /// Free extra functions for implementer of [ApexBufferP1]
    pub trait ApexBufferP1Ext: ApexBufferP1 + Sized {
        /// # Errors
        /// - [Error::InvalidConfig]: buffer with `name` does not exist
        fn get_buffer(name: Name) -> Result<Buffer<Self>, Error>;
    }

    impl<B: ApexBufferP1> ApexBufferP1Ext for B {
        fn get_buffer(name: Name) -> Result<Buffer<B>, Error> {
            let id = B::get_buffer_id(name.into())?;
            // According to ARINC653P1-5 3.7.2.1.5 this can only fail if the buffer_id
            //  does not exist in the current partition.
            // But since we retrieve the buffer_id directly from the hypervisor
            //  there is no possible way for it not existing
            let status = B::get_buffer_status(id).unwrap();

            Ok(Buffer {
                _b: Default::default(),
                id,
                max_size: status.max_message_size,
                max_number_msgs: status.max_nb_message,
            })
        }
    }

    impl<B: ApexBufferP1> Buffer<B> {
        /// # Errors
        /// - [Error::InvalidConfig]: buffer with `name` does not exist
        pub fn from_name(name: Name) -> Result<Buffer<B>, Error> {
            B::get_buffer(name)
        }

        pub fn id(&self) -> BufferId {
            self.id
        }

        /// Max [MessageSize] for this [Buffer]
        pub fn size(&self) -> MessageSize {
            self.max_size
        }

        /// Max number of messages in this [Buffer]
        pub fn range(&self) -> MessageRange {
            self.max_number_msgs
        }

        /// Checked Buffer send from specified byte buffer
        ///
        /// # Errors
        /// - [Error::WriteError]: the `buffer` is longer than the `max_message_size` specified for this buffer
        /// - [Error::WriteError]: `buffer` length is zero
        pub fn send(&self, buffer: &mut [ApexByte], timeout: SystemTime) -> Result<(), Error> {
            buffer.validate_write(self.max_size)?;
            B::send_buffer(self.id, buffer, timeout.into())?;
            Ok(())
        }

        /// Checked Buffer receive into specified byte buffer
        ///
        /// # Errors
        /// - [Error::InvalidParam]: `timeout` is invalid
        /// - [Error::InvalidMode]: current process holds a mutex
        /// - [Error::InvalidMode]: current process is error handler AND `timeout` is not instant.
        /// - [Error::NotAvailable]: there is no message in the buffer
        /// - [Error::TimedOut]: `timeout` elapsed
        /// - [Error::ReadError]: prodived `buffer` is too small for this [Buffer]'s `max_message_size`
        pub fn receive<'a>(
            &self,
            buffer: &'a mut [ApexByte],
            timeout: SystemTime,
        ) -> Result<&'a [ApexByte], Error> {
            buffer.validate_read(self.max_size)?;
            unsafe { self.receive_unchecked(timeout, buffer) }
        }

        /// Unchecked Buffer receive into specified byte buffer
        ///
        /// # Errors
        /// - [Error::InvalidParam]: `timeout` is invalid
        /// - [Error::InvalidMode]: current process holds a mutex
        /// - [Error::InvalidMode]: current process is error handler AND `timeout` is not instant.
        /// - [Error::NotAvailable]: there is no message in the buffer
        /// - [Error::TimedOut]: `timeout` elapsed
        ///
        /// # Safety
        ///
        /// This function is safe, as long as the buffer can hold whatever is received
        pub unsafe fn receive_unchecked<'a>(
            &self,
            timeout: SystemTime,
            buffer: &'a mut [ApexByte],
        ) -> Result<&'a [ApexByte], Error> {
            let len = B::receive_buffer(self.id, timeout.into(), buffer)? as usize;
            Ok(&buffer[..len])
        }

        /// # Panics
        /// if this buffer does not exist anymore
        pub fn status(&self) -> BufferStatus {
            // According to ARINC653P1-5 3.7.2.1.5 this can only fail if the buffer_id
            //  does not exist in the current partition.
            // But since we retrieve the buffer_id directly from the hypervisor
            //  there is no possible way for it not existing
            B::get_buffer_status(self.id).unwrap()
        }
    }

    impl<B: ApexBufferP1> StartContext<B> {
        /// # Errors
        /// - [Error::InvalidConfig]: not enough memory is available
        /// - [Error::InvalidConfig]: [ApexLimits::SYSTEM_LIMIT_NUMBER_OF_BUFFERS](crate::apex::limits::ApexLimits::SYSTEM_LIMIT_NUMBER_OF_BUFFERS) was reached
        /// - [Error::NoAction]: a buffer with given `name` already exists
        /// - [Error::InvalidParam]: `size` is zero
        /// - [Error::InvalidParam]: `range` is too large
        pub fn create_buffer(
            &mut self,
            name: Name,
            size: MessageSize,
            range: MessageRange,
            qd: QueuingDiscipline,
        ) -> Result<Buffer<B>, Error> {
            let id = B::create_buffer(name.into(), size, range, qd)?;
            Ok(Buffer {
                _b: Default::default(),
                id,
                max_size: size,
                max_number_msgs: range,
            })
        }
    }
}
