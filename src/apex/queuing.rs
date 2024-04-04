/// bindings for ARINC653P1-5 3.6.2.2 queuing
pub mod basic {
    use crate::apex::time::basic::*;
    use crate::apex::types::basic::*;

    pub type QueuingPortName = ApexName;

    /// According to ARINC 653P1-5 this may either be 32 or 64 bits.
    /// Internally we will use 64-bit by default.
    /// The implementing Hypervisor may cast this to 32-bit if needed
    pub type QueuingPortId = ApexLongInteger;

    /// The queue overflowed on the sender side
    pub type QueueOverflow = bool;

    /// ARINC 653P1-5 3.6.2.2.3 states that [ErrorReturnCode::InvalidConfig] signals that the queue overflowed on the sender side
    #[cfg_attr(not(feature = "bindings"), allow(dead_code))]
    pub const QUEUE_OVERFLOW_ERROR: ErrorReturnCode = ErrorReturnCode::InvalidConfig;

    #[derive(Debug, Clone, PartialEq, Eq)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
    pub struct QueuingPortStatus {
        pub nb_message: MessageRange,
        pub max_nb_message: MessageRange,
        pub max_message_size: MessageSize,
        pub port_direction: PortDirection,
        pub waiting_processes: WaitingRange,
    }

    pub trait ApexQueuingPortP4 {
        // Only during Warm/Cold-Start
        fn create_queuing_port(
            queuing_port_name: QueuingPortName,
            max_message_size: MessageSize,
            max_nb_message: MessageRange,
            port_direction: PortDirection,
            queuing_discipline: QueuingDiscipline,
        ) -> Result<QueuingPortId, ErrorReturnCode>;

        fn send_queuing_message(
            queuing_port_id: QueuingPortId,
            message: &[ApexByte],
            time_out: ApexSystemTime,
        ) -> Result<(), ErrorReturnCode>;

        /// # Safety
        ///
        /// This function is safe, as long as the buffer can hold whatever is received
        unsafe fn receive_queuing_message(
            queuing_port_id: QueuingPortId,
            time_out: ApexSystemTime,
            message: &mut [ApexByte],
        ) -> Result<(MessageSize, QueueOverflow), ErrorReturnCode>;

        fn get_queuing_port_status(
            queuing_port_id: QueuingPortId,
        ) -> Result<QueuingPortStatus, ErrorReturnCode>;

        fn clear_queuing_port(queuing_port_id: QueuingPortId) -> Result<(), ErrorReturnCode>;
    }

    pub trait ApexQueuingPortP1: ApexQueuingPortP4 {
        fn get_queuing_port_id(
            queuing_port_name: QueuingPortName,
        ) -> Result<QueuingPortId, ErrorReturnCode>;
    }
}

/// abstractions for ARINC653P1-5 3.6.2.2 queuing
pub mod abstraction {
    use core::marker::PhantomData;
    use core::sync::atomic::AtomicPtr;

    use super::basic::{ApexQueuingPortP1, ApexQueuingPortP4};
    // Reexport important basic-types for downstream-user
    pub use super::basic::{QueueOverflow, QueuingPortId, QueuingPortStatus};
    use crate::apex::types::basic::PortDirection;
    use crate::prelude::*;

    #[derive(Debug)]
    pub struct ConstQueuingPortSender<
        const MSG_SIZE: MessageSize,
        const NB_MSGS: MessageRange,
        Q: ApexQueuingPortP4Ext,
    > {
        _b: PhantomData<AtomicPtr<Q>>,
        id: QueuingPortId,
    }

    impl<const MSG_SIZE: MessageSize, const NB_MSGS: MessageRange, S: ApexQueuingPortP4Ext> Clone
        for ConstQueuingPortSender<MSG_SIZE, NB_MSGS, S>
    {
        fn clone(&self) -> Self {
            Self {
                _b: self._b,
                id: self.id,
            }
        }
    }

    #[derive(Debug)]
    pub struct ConstQueuingPortReceiver<
        const MSG_SIZE: MessageSize,
        const NB_MSGS: MessageRange,
        Q: ApexQueuingPortP4Ext,
    > {
        _b: PhantomData<AtomicPtr<Q>>,
        id: QueuingPortId,
    }

    impl<const MSG_SIZE: MessageSize, const NB_MSGS: MessageRange, S: ApexQueuingPortP4Ext> Clone
        for ConstQueuingPortReceiver<MSG_SIZE, NB_MSGS, S>
    {
        fn clone(&self) -> Self {
            Self {
                _b: self._b,
                id: self.id,
            }
        }
    }

    pub trait ApexQueuingPortP4Ext: ApexQueuingPortP4 + Sized {
        fn queueing_port_send_unchecked(
            id: QueuingPortId,
            buffer: &[ApexByte],
            timeout: SystemTime,
        ) -> Result<(), Error>;

        /// # Safety
        ///
        /// This function is safe, as long as the buffer can hold whatever is received
        unsafe fn queueing_port_receive_unchecked(
            id: QueuingPortId,
            timeout: SystemTime,
            buffer: &mut [ApexByte],
        ) -> Result<(&[ApexByte], QueueOverflow), Error>;
    }

    pub trait ApexQueuingPortP1Ext: ApexQueuingPortP1 + Sized {
        /// Returns Err(Error::InvalidConfig) if queuing port with name does not exists or
        /// if the message size of the found queuing port is different than MSG_SIZE or
        /// if number messages of found queuing port is different than NB_MSGS
        fn get_const_queuing_port_sender<const MSG_SIZE: MessageSize, const NB_MSGS: MessageRange>(
            name: Name,
        ) -> Result<ConstQueuingPortSender<MSG_SIZE, NB_MSGS, Self>, Error>;

        /// Returns Err(Error::InvalidConfig) if queuing port with name does not exists or
        /// if the message size of the found queuing port is different than MSG_SIZE or
        /// if number messages of found queuing port is different than NB_MSGS
        fn get_const_queuing_port_receiver<
            const MSG_SIZE: MessageSize,
            const NB_MSGS: MessageRange,
        >(
            name: Name,
        ) -> Result<ConstQueuingPortReceiver<MSG_SIZE, NB_MSGS, Self>, Error>;
    }

    impl<Q: ApexQueuingPortP4> ApexQueuingPortP4Ext for Q {
        fn queueing_port_send_unchecked(
            id: QueuingPortId,
            buffer: &[ApexByte],
            timeout: SystemTime,
        ) -> Result<(), Error> {
            Q::send_queuing_message(id, buffer, timeout.into())?;
            Ok(())
        }

        unsafe fn queueing_port_receive_unchecked(
            id: QueuingPortId,
            timeout: SystemTime,
            buffer: &mut [ApexByte],
        ) -> Result<(&[ApexByte], QueueOverflow), Error> {
            let (len, overflow) = Q::receive_queuing_message(id, timeout.into(), buffer)?;
            Ok((&buffer[..(len as usize)], overflow))
        }
    }

    impl<Q: ApexQueuingPortP1> ApexQueuingPortP1Ext for Q {
        /// Returns Err(Error::InvalidConfig) if queuing port with name does not exists or
        /// if the message size of the found queuing port is different than MSG_SIZE or
        /// if number messages of found queuing port is different than NB_MSGS
        fn get_const_queuing_port_sender<
            const MSG_SIZE: MessageSize,
            const NB_MSGS: MessageRange,
        >(
            name: Name,
        ) -> Result<ConstQueuingPortSender<MSG_SIZE, NB_MSGS, Q>, Error> {
            let id = Q::get_queuing_port_id(name.into())?;
            // According to ARINC653P1-5 3.6.2.2.5 this can only fail if the queuing_port_id
            //  does not exist in the current partition.
            // But since we retrieve the queuing_port_id directly from the hypervisor
            //  there is no possible way for it not existing
            let QueuingPortStatus {
                max_nb_message,
                max_message_size,
                port_direction,
                ..
            } = Q::get_queuing_port_status(id).unwrap();

            if max_nb_message != NB_MSGS {
                return Err(Error::InvalidConfig);
            }

            if max_message_size != MSG_SIZE {
                return Err(Error::InvalidConfig);
            }

            if port_direction != PortDirection::Source {
                return Err(Error::InvalidConfig);
            }

            Ok(ConstQueuingPortSender {
                _b: Default::default(),
                id,
            })
        }

        /// Returns Err(Error::InvalidConfig) if queuing port with name does not exists or
        /// if the message size of the found queuing port is different than MSG_SIZE or
        /// if number messages of found queuing port is different than NB_MSGS
        fn get_const_queuing_port_receiver<
            const MSG_SIZE: MessageSize,
            const NB_MSGS: MessageRange,
        >(
            name: Name,
        ) -> Result<ConstQueuingPortReceiver<MSG_SIZE, NB_MSGS, Q>, Error> {
            let id = Q::get_queuing_port_id(name.into())?;
            // According to ARINC653P1-5 3.6.2.2.5 this can only fail if the queuing_port_id
            //  does not exist in the current partition.
            // But since we retrieve the queuing_port_id directly from the hypervisor
            //  there is no possible way for it not existing
            let QueuingPortStatus {
                max_nb_message,
                max_message_size,
                port_direction,
                ..
            } = Q::get_queuing_port_status(id).unwrap();

            if max_nb_message != NB_MSGS {
                return Err(Error::InvalidConfig);
            }

            if max_message_size != MSG_SIZE {
                return Err(Error::InvalidConfig);
            }

            if port_direction != PortDirection::Destination {
                return Err(Error::InvalidConfig);
            }

            Ok(ConstQueuingPortReceiver {
                _b: Default::default(),
                id,
            })
        }
    }

    impl<const MSG_SIZE: MessageSize, const NB_MSGS: MessageRange, Q: ApexQueuingPortP4Ext>
        ConstQueuingPortSender<MSG_SIZE, NB_MSGS, Q>
    {
        pub fn send(&self, buffer: &[ApexByte], timeout: SystemTime) -> Result<(), Error> {
            buffer.validate_write(MSG_SIZE)?;
            Q::queueing_port_send_unchecked(self.id, buffer, timeout)
        }

        pub fn id(&self) -> QueuingPortId {
            self.id
        }

        pub const fn size(&self) -> usize {
            MSG_SIZE as usize
        }

        pub const fn range(&self) -> MessageRange {
            NB_MSGS
        }

        pub fn status(&self) -> QueuingPortStatus {
            // According to ARINC653P1-5 3.6.2.2.5 this can only fail if the queuing_port_id
            //  does not exist in the current partition.
            // But since we retrieve the queuing_port_id directly from the hypervisor
            //  there is no possible way for it not existing
            Q::get_queuing_port_status(self.id).unwrap()
        }
    }

    impl<const MSG_SIZE: MessageSize, const NB_MSGS: MessageRange, Q: ApexQueuingPortP1Ext>
        ConstQueuingPortSender<MSG_SIZE, NB_MSGS, Q>
    {
        pub fn from_name(
            name: Name,
        ) -> Result<ConstQueuingPortSender<MSG_SIZE, NB_MSGS, Q>, Error> {
            Q::get_const_queuing_port_sender(name)
        }
    }

    impl<const MSG_SIZE: MessageSize, const NB_MSGS: MessageRange, Q: ApexQueuingPortP4Ext>
        ConstQueuingPortReceiver<MSG_SIZE, NB_MSGS, Q>
    {
        pub fn receive<'a>(
            &self,
            buffer: &'a mut [ApexByte],
            timeout: SystemTime,
        ) -> Result<(&'a [ApexByte], QueueOverflow), Error> {
            buffer.validate_read(MSG_SIZE)?;
            unsafe { Q::queueing_port_receive_unchecked(self.id, timeout, buffer) }
        }

        pub fn clear(&self) {
            // According to ARINC653P1-5 3.6.2.2.6 this can only fail if the queuing_port_id does not exist
            //  in the current partition or if this is not a destination port.
            // But since we retrieve the queuing_port_id directly from the hypervisor
            //  and we verify that this is a destination port,
            //  there is no possible way for it not existing
            Q::clear_queuing_port(self.id).unwrap();
        }

        pub fn id(&self) -> QueuingPortId {
            self.id
        }

        pub const fn size(&self) -> usize {
            MSG_SIZE as usize
        }

        pub const fn range(&self) -> MessageRange {
            NB_MSGS
        }

        pub fn status(&self) -> QueuingPortStatus {
            // According to ARINC653P1-5 3.6.2.2.5 this can only fail if the queuing_port_id
            //  does not exist in the current partition.
            // But since we retrieve the queuing_port_id directly from the hypervisor
            //  there is no possible way for it not existing
            Q::get_queuing_port_status(self.id).unwrap()
        }
    }

    impl<const MSG_SIZE: MessageSize, const NB_MSGS: MessageRange, Q: ApexQueuingPortP1Ext>
        ConstQueuingPortReceiver<MSG_SIZE, NB_MSGS, Q>
    {
        pub fn from_name(
            name: Name,
        ) -> Result<ConstQueuingPortReceiver<MSG_SIZE, NB_MSGS, Q>, Error> {
            Q::get_const_queuing_port_receiver(name)
        }
    }

    impl<Q: ApexQueuingPortP4Ext> StartContext<Q> {
        pub fn create_const_queuing_port_sender<
            const MSG_SIZE: MessageSize,
            const NB_MSGS: MessageRange,
        >(
            &mut self,
            name: Name,
            qd: QueuingDiscipline,
        ) -> Result<ConstQueuingPortSender<MSG_SIZE, NB_MSGS, Q>, Error> {
            let id =
                Q::create_queuing_port(name.into(), MSG_SIZE, NB_MSGS, PortDirection::Source, qd)?;

            Ok(ConstQueuingPortSender {
                _b: Default::default(),
                id,
            })
        }

        pub fn create_const_queuing_port_receiver<
            const MSG_SIZE: MessageSize,
            const NB_MSGS: MessageRange,
        >(
            &mut self,
            name: Name,
            qd: QueuingDiscipline,
        ) -> Result<ConstQueuingPortReceiver<MSG_SIZE, NB_MSGS, Q>, Error> {
            let id = Q::create_queuing_port(
                name.into(),
                MSG_SIZE,
                NB_MSGS,
                PortDirection::Destination,
                qd,
            )?;

            Ok(ConstQueuingPortReceiver {
                _b: Default::default(),
                id,
            })
        }
    }
}
