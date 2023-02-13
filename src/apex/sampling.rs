pub mod basic {
    use crate::bindings::*;
    use crate::Locked;

    pub type SamplingPortName = ApexName;

    // TODO P2 extension

    /// According to ARINC 653P1-5 this may either be 32 or 64 bits.
    /// Internally we will use 64-bit by default.
    /// The implementing Hypervisor may cast this to 32-bit if needed
    pub type SamplingPortId = ApexLongInteger;

    #[repr(u32)]
    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
    #[cfg_attr(feature = "strum", derive(strum::FromRepr))]
    pub enum Validity {
        Invalid = 0,
        Valid = 1,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
    pub struct ApexSamplingPortStatus {
        pub refresh_period: ApexSystemTime,
        pub max_message_size: MessageSize,
        pub port_direction: PortDirection,
        pub last_msg_validity: Validity,
    }

    pub trait ApexSamplingPortP4 {
        // Only during Warm/Cold-Start
        fn create_sampling_port<L: Locked>(
            sampling_port_name: SamplingPortName,
            max_message_size: MessageSize,
            port_direction: PortDirection,
            refresh_period: ApexSystemTime,
        ) -> Result<SamplingPortId, ErrorReturnCode>;

        fn write_sampling_message<L: Locked>(
            sampling_port_id: SamplingPortId,
            message: &[ApexByte],
        ) -> Result<(), ErrorReturnCode>;

        /// # Safety
        ///
        /// This function is safe, as long as the buffer can hold whatever is received
        unsafe fn read_sampling_message<L: Locked>(
            sampling_port_id: SamplingPortId,
            message: &mut [ApexByte],
        ) -> Result<(Validity, MessageSize), ErrorReturnCode>;
    }

    pub trait ApexSamplingPortP1: ApexSamplingPortP4 {
        fn get_sampling_port_id<L: Locked>(
            sampling_port_name: SamplingPortName,
        ) -> Result<SamplingPortId, ErrorReturnCode>;

        fn get_sampling_port_status<L: Locked>(
            sampling_port_id: SamplingPortId,
        ) -> Result<ApexSamplingPortStatus, ErrorReturnCode>;
    }
}
pub mod abstraction {
    use core::marker::PhantomData;
    use core::time::Duration;

    // Reexport important basic-types for downstream-user
    pub use super::basic::{ApexSamplingPortP1, ApexSamplingPortP4, SamplingPortId, Validity};
    use crate::bindings::*;
    use crate::hidden::Key;
    use crate::prelude::*;

    #[derive(Debug, Clone, PartialEq, Eq)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
    pub struct SamplingPortStatus {
        pub refresh_period: SystemTime,
        pub max_message_size: MessageSize,
        pub port_direction: PortDirection,
        pub last_msg_validity: Validity,
    }

    impl From<ApexSamplingPortStatus> for SamplingPortStatus {
        fn from(s: ApexSamplingPortStatus) -> Self {
            SamplingPortStatus {
                refresh_period: s.refresh_period.into(),
                max_message_size: s.max_message_size,
                port_direction: s.port_direction,
                last_msg_validity: s.last_msg_validity,
            }
        }
    }

    #[derive(Debug, Clone)]
    pub struct SamplingPortSource<const MSG_SIZE: MessageSize, S: ApexSamplingPortP4> {
        _b: PhantomData<S>,
        id: SamplingPortId,
    }

    #[derive(Debug, Clone)]
    pub struct SamplingPortDestination<const MSG_SIZE: MessageSize, S: ApexSamplingPortP4> {
        _b: PhantomData<S>,
        id: SamplingPortId,
        refresh: Duration,
    }

    pub trait ApexSamplingPortP4Ext: ApexSamplingPortP4 + Sized {
        fn sampling_port_send_unchecked(
            id: SamplingPortId,
            buffer: &[ApexByte],
        ) -> Result<(), Error>;

        /// # Safety
        ///
        /// This function is safe, as long as the buffer can hold whatever is received
        unsafe fn sampling_port_receive_unchecked(
            id: SamplingPortId,
            buffer: &mut [ApexByte],
        ) -> Result<(Validity, &[ApexByte]), Error>;
    }

    pub trait ApexSamplingPortP1Ext: ApexSamplingPortP1 + Sized {
        /// Returns Err(Error::InvalidConfig) if sampling port with name does not exists or
        /// if the message size of the found sampling port is different than MSG_SIZE
        fn get_sampling_port_source<const MSG_SIZE: MessageSize>(
            name: Name,
        ) -> Result<SamplingPortSource<MSG_SIZE, Self>, Error>;

        /// Returns Err(Error::InvalidConfig) if sampling port with name does not exists or
        /// if the message size of the found sampling port is different than MSG_SIZE
        fn get_sampling_port_destination<const MSG_SIZE: MessageSize>(
            name: Name,
        ) -> Result<SamplingPortDestination<MSG_SIZE, Self>, Error>;
    }

    impl<S: ApexSamplingPortP4> ApexSamplingPortP4Ext for S {
        fn sampling_port_send_unchecked(
            id: SamplingPortId,
            buffer: &[ApexByte],
        ) -> Result<(), Error> {
            S::write_sampling_message::<Key>(id, buffer)?;
            Ok(())
        }

        unsafe fn sampling_port_receive_unchecked(
            id: SamplingPortId,
            buffer: &mut [ApexByte],
        ) -> Result<(Validity, &[ApexByte]), Error> {
            let (val, len) = S::read_sampling_message::<Key>(id, buffer)?;
            Ok((val, &buffer[..(len as usize)]))
        }
    }

    impl<S: ApexSamplingPortP1> ApexSamplingPortP1Ext for S {
        /// Returns Err(Error::InvalidConfig) if sampling port with name does not exists or
        /// if the message size of the found sampling port is different than MSG_SIZE
        fn get_sampling_port_source<const MSG_SIZE: MessageSize>(
            name: Name,
        ) -> Result<SamplingPortSource<MSG_SIZE, Self>, Error> {
            let id = S::get_sampling_port_id::<Key>(name.into())?;
            // According to ARINC653P1-5 3.6.2.1.5 this can only fail if the sampling_port_id
            //  does not exist in the current partition.
            // But since we retrieve the sampling_port_id directly from the hypervisor
            //  there is no possible way for it not existing
            let SamplingPortStatus {
                refresh_period: _,
                max_message_size,
                port_direction,
                ..
            } = S::get_sampling_port_status::<Key>(id).unwrap().into();

            if max_message_size != MSG_SIZE {
                return Err(Error::InvalidConfig);
            }

            if port_direction != PortDirection::Source {
                return Err(Error::InvalidConfig);
            }

            Ok(SamplingPortSource {
                _b: Default::default(),
                id,
            })
        }

        /// Returns Err(Error::InvalidConfig) if sampling port with name does not exists or
        /// if the message size of the found sampling port is different than MSG_SIZE
        fn get_sampling_port_destination<const MSG_SIZE: MessageSize>(
            name: Name,
        ) -> Result<SamplingPortDestination<MSG_SIZE, Self>, Error> {
            let id = S::get_sampling_port_id::<Key>(name.into())?;
            // According to ARINC653P1-5 3.6.2.1.5 this can only fail if the sampling_port_id
            //  does not exist in the current partition.
            // But since we retrieve the sampling_port_id directly from the hypervisor
            //  there is no possible way for it not existing
            let SamplingPortStatus {
                refresh_period,
                max_message_size,
                port_direction,
                ..
            } = S::get_sampling_port_status::<Key>(id)?.into();

            if max_message_size != MSG_SIZE {
                return Err(Error::InvalidConfig);
            }

            if port_direction != PortDirection::Destination {
                return Err(Error::InvalidConfig);
            }

            Ok(SamplingPortDestination {
                _b: Default::default(),
                id,
                // According to ARINC653P1-5 3.6.2.1.1 the refresh_period defined during
                //  COLD/WARM-Start is always positive, hence this unwrap cannot fail
                refresh: refresh_period.unwrap_duration(),
            })
        }
    }

    impl<const MSG_SIZE: MessageSize, S: ApexSamplingPortP4> SamplingPortSource<MSG_SIZE, S> {
        pub fn send(&self, buffer: &[ApexByte]) -> Result<(), Error> {
            WriteError::validate(MSG_SIZE, buffer)?;
            S::sampling_port_send_unchecked(self.id, buffer)
        }

        pub fn id(&self) -> SamplingPortId {
            self.id
        }

        pub const fn size(&self) -> MessageSize {
            MSG_SIZE
        }
    }

    impl<const MSG_SIZE: MessageSize, S: ApexSamplingPortP1> SamplingPortSource<MSG_SIZE, S> {
        pub fn from_name(name: Name) -> Result<SamplingPortSource<MSG_SIZE, S>, Error> {
            S::get_sampling_port_source(name)
        }

        pub fn status(&self) -> SamplingPortStatus {
            // According to ARINC653P1-5 3.6.2.1.5 this can only fail if the sampling_port_id
            //  does not exist in the current partition.
            // But since we retrieve the sampling_port_id directly from the hypervisor
            //  there is no possible way for it not existing
            S::get_sampling_port_status::<Key>(self.id).unwrap().into()
        }
    }

    impl<const MSG_SIZE: MessageSize, S: ApexSamplingPortP4> SamplingPortDestination<MSG_SIZE, S> {
        pub fn receive<'a>(
            &self,
            buffer: &'a mut [ApexByte],
        ) -> Result<(Validity, &'a [ApexByte]), Error> {
            ReadError::validate(MSG_SIZE, buffer)?;
            unsafe { S::sampling_port_receive_unchecked(self.id, buffer) }
        }

        pub fn id(&self) -> SamplingPortId {
            self.id
        }

        pub const fn size(&self) -> MessageSize {
            MSG_SIZE
        }

        pub fn refresh_period(&self) -> Duration {
            self.refresh
        }
    }

    impl<const MSG_SIZE: MessageSize, S: ApexSamplingPortP1> SamplingPortDestination<MSG_SIZE, S> {
        pub fn from_name(name: Name) -> Result<SamplingPortDestination<MSG_SIZE, S>, Error> {
            S::get_sampling_port_destination(name)
        }

        pub fn status(&self) -> SamplingPortStatus {
            // According to ARINC653P1-5 3.6.2.1.5 this can only fail if the sampling_port_id
            //  does not exist in the current partition.
            // But since we retrieve the sampling_port_id directly from the hypervisor
            //  there is no possible way for it not existing
            S::get_sampling_port_status::<Key>(self.id).unwrap().into()
        }
    }

    impl<S: ApexSamplingPortP4> StartContext<S> {
        pub fn create_sampling_port_source<const MSG_SIZE: MessageSize>(
            &mut self,
            name: Name,
        ) -> Result<SamplingPortSource<MSG_SIZE, S>, Error> {
            let id = S::create_sampling_port::<Key>(
                name.into(),
                MSG_SIZE,
                PortDirection::Source,
                // use random non-zero duration.
                // while refresh_period is ignored for the source
                //  it may produce an error if set to zero
                // XNG 1.4 needs 2000 nanos for some reason
                SystemTime::Normal(Duration::from_nanos(2000)).into(),
            )?;
            Ok(SamplingPortSource {
                _b: Default::default(),
                id,
            })
        }
        pub fn create_sampling_port_destination<const MSG_SIZE: MessageSize>(
            &mut self,
            name: Name,
            refresh: Duration,
        ) -> Result<SamplingPortDestination<MSG_SIZE, S>, Error> {
            let id = S::create_sampling_port::<Key>(
                name.into(),
                MSG_SIZE,
                PortDirection::Destination,
                SystemTime::Normal(refresh).into(),
            )?;
            Ok(SamplingPortDestination {
                _b: Default::default(),
                id,
                refresh,
            })
        }
    }
}
