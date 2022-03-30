pub mod basic {
    use crate::bindings::*;
    use crate::Locked;

    pub type BufferName = ApexName;

    /// According to ARINC 653P1-5 this may either be 32 or 64 bits.
    /// Internally we will use 64-bit by default.
    /// The implementing Hypervisor may cast this to 32-bit if needed
    pub type BufferId = ApexLongInteger;

    pub trait ApexBufferP1 {
        // Only during Warm/Cold-Start
        fn create_buffer<L: Locked>(
            buffer_name: BufferName,
            max_message_size: MessageSize,
            max_nb_message: MessageRange,
            queuing_discipline: QueuingDiscipline,
        ) -> Result<BufferId, ErrorReturnCode>;

        fn send_buffer<L: Locked>(
            buffer_id: BufferId,
            message: &[ApexByte],
            time_out: ApexSystemTime,
        ) -> Result<(), ErrorReturnCode>;

        /// # Safety
        ///
        /// This function is safe, as long as the buffer can hold whatever is received
        unsafe fn receive_buffer<L: Locked>(
            buffer_id: BufferId,
            time_out: ApexSystemTime,
            message: &mut [ApexByte],
        ) -> Result<MessageSize, ErrorReturnCode>;

        fn get_buffer_id<L: Locked>(buffer_name: BufferName) -> Result<BufferId, ErrorReturnCode>;

        fn get_buffer_status<L: Locked>(
            buffer_id: BufferId,
        ) -> Result<BufferStatus, ErrorReturnCode>;
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct BufferStatus {
        pub nb_message: MessageRange,
        pub max_nb_message: MessageRange,
        pub max_message_size: MessageSize,
        pub waiting_processes: WaitingRange,
    }
}
pub mod abstraction {
    use core::marker::PhantomData;

    // Reexport important basic-types for downstream-user
    pub use super::basic::{ApexBufferP1, BufferId, BufferStatus};
    use crate::bindings::*;
    use crate::hidden::Key;
    use crate::prelude::*;

    #[derive(Debug, Clone)]
    pub struct Buffer<B: ApexBufferP1> {
        _b: PhantomData<B>,
        id: BufferId,
        size: MessageSize,
        range: MessageRange,
    }

    pub trait ApexBufferP1Ext: ApexBufferP1 + Sized {
        fn get_buffer(name: Name) -> Result<Buffer<Self>, Error>;
    }

    impl<B: ApexBufferP1> ApexBufferP1Ext for B {
        fn get_buffer(name: Name) -> Result<Buffer<B>, Error> {
            let id = B::get_buffer_id::<Key>(name.into())?;
            // According to ARINC653P1-5 3.7.2.1.5 this can only fail if the buffer_id
            //  does not exist in the current partition.
            // But since we retrieve the buffer_id directly from the hypervisor
            //  there is no possible way for it not existing
            let status = B::get_buffer_status::<Key>(id).unwrap();

            Ok(Buffer {
                _b: Default::default(),
                id,
                size: status.max_message_size,
                range: status.max_nb_message,
            })
        }
    }

    impl<B: ApexBufferP1> Buffer<B> {
        pub fn from_name(name: Name) -> Result<Buffer<B>, Error> {
            B::get_buffer(name)
        }

        pub fn id(&self) -> BufferId {
            self.id
        }

        pub fn size(&self) -> MessageSize {
            self.size
        }

        pub fn range(&self) -> MessageRange {
            self.range
        }

        pub fn send(&self, buffer: &mut [ApexByte], timeout: SystemTime) -> Result<(), Error> {
            WriteError::validate(self.size, buffer)?;
            B::send_buffer::<Key>(self.id, buffer, timeout.into())?;
            Ok(())
        }

        pub fn receive<'a>(
            &self,
            buffer: &'a mut [ApexByte],
            timeout: SystemTime,
        ) -> Result<&'a [ApexByte], Error> {
            ReadError::validate(self.size, buffer)?;
            unsafe { self.receive_unchecked(timeout, buffer) }
        }

        /// # Safety
        ///
        /// This function is safe, as long as the buffer can hold whatever is received
        pub unsafe fn receive_unchecked<'a>(
            &self,
            timeout: SystemTime,
            buffer: &'a mut [ApexByte],
        ) -> Result<&'a [ApexByte], Error> {
            let len = B::receive_buffer::<Key>(self.id, timeout.into(), buffer)? as usize;
            Ok(&buffer[..len])
        }

        pub fn status(&self) -> BufferStatus {
            // According to ARINC653P1-5 3.7.2.1.5 this can only fail if the buffer_id
            //  does not exist in the current partition.
            // But since we retrieve the buffer_id directly from the hypervisor
            //  there is no possible way for it not existing
            B::get_buffer_status::<Key>(self.id).unwrap()
        }
    }

    impl<B: ApexBufferP1> StartContext<B> {
        pub fn create_buffer(
            &mut self,
            name: Name,
            size: MessageSize,
            range: MessageRange,
            qd: QueuingDiscipline,
        ) -> Result<Buffer<B>, Error> {
            let id = B::create_buffer::<Key>(name.into(), size, range, qd)?;
            Ok(Buffer {
                _b: Default::default(),
                id,
                size,
                range,
            })
        }
    }
}
