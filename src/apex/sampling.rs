/// bindings for ARINC653P1-5 3.6.2.1 sampling
pub mod basic {
    use crate::apex::time::basic::*;
    use crate::apex::types::basic::*;

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
        fn create_sampling_port(
            sampling_port_name: SamplingPortName,
            max_message_size: MessageSize,
            port_direction: PortDirection,
            refresh_period: ApexSystemTime,
        ) -> Result<SamplingPortId, ErrorReturnCode>;

        fn write_sampling_message(
            sampling_port_id: SamplingPortId,
            message: &[ApexByte],
        ) -> Result<(), ErrorReturnCode>;

        /// # Safety
        ///
        /// This function is safe, as long as the buffer can hold whatever is received
        unsafe fn read_sampling_message(
            sampling_port_id: SamplingPortId,
            message: &mut [ApexByte],
        ) -> Result<(Validity, MessageSize), ErrorReturnCode>;
    }

    pub trait ApexSamplingPortP1: ApexSamplingPortP4 {
        fn get_sampling_port_id(
            sampling_port_name: SamplingPortName,
        ) -> Result<SamplingPortId, ErrorReturnCode>;

        fn get_sampling_port_status(
            sampling_port_id: SamplingPortId,
        ) -> Result<ApexSamplingPortStatus, ErrorReturnCode>;
    }
}

/// abstractions for ARINC653P1-5 3.6.2.1 sampling
pub mod abstraction {
    use core::borrow::Borrow;
    use core::marker::PhantomData;
    use core::ops::Deref;
    use core::sync::atomic::AtomicPtr;
    use core::time::Duration;

    use super::basic::{ApexSamplingPortP1, ApexSamplingPortP4, ApexSamplingPortStatus};
    // Reexport important basic-types for downstream-user
    pub use super::basic::{SamplingPortId, Validity};
    use crate::apex::types::basic::PortDirection;
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
    pub struct ConstSamplingPortSource<const MSG_SIZE: MessageSize, S: ApexSamplingPortP4Ext>(
        SamplingPortSource<S>,
    );

    impl<const MSG_SIZE: MessageSize, S: ApexSamplingPortP4Ext> Deref
        for ConstSamplingPortSource<MSG_SIZE, S>
    {
        type Target = SamplingPortSource<S>;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    impl<const MSG_SIZE: MessageSize, S: ApexSamplingPortP4Ext> AsRef<SamplingPortSource<S>>
        for ConstSamplingPortSource<MSG_SIZE, S>
    {
        fn as_ref(&self) -> &SamplingPortSource<S> {
            &self.0
        }
    }

    impl<const MSG_SIZE: MessageSize, S: ApexSamplingPortP4Ext> Borrow<SamplingPortSource<S>>
        for ConstSamplingPortSource<MSG_SIZE, S>
    {
        fn borrow(&self) -> &SamplingPortSource<S> {
            &self.0
        }
    }

    impl<const MSG_SIZE: MessageSize, S: ApexSamplingPortP4Ext> TryFrom<SamplingPortSource<S>>
        for ConstSamplingPortSource<MSG_SIZE, S>
    {
        type Error = Error;

        fn try_from(port: SamplingPortSource<S>) -> Result<Self, Self::Error> {
            if port.msg_size != MSG_SIZE {
                return Err(Error::InvalidConfig);
            }

            Ok(ConstSamplingPortSource(port))
        }
    }

    #[derive(Debug, Clone)]
    pub struct ConstSamplingPortDestination<const MSG_SIZE: MessageSize, S: ApexSamplingPortP4Ext>(
        SamplingPortDestination<S>,
    );

    impl<const MSG_SIZE: MessageSize, S: ApexSamplingPortP4Ext> Deref
        for ConstSamplingPortDestination<MSG_SIZE, S>
    {
        type Target = SamplingPortDestination<S>;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    impl<const MSG_SIZE: MessageSize, S: ApexSamplingPortP4Ext> AsRef<SamplingPortDestination<S>>
        for ConstSamplingPortDestination<MSG_SIZE, S>
    {
        fn as_ref(&self) -> &SamplingPortDestination<S> {
            &self.0
        }
    }

    impl<const MSG_SIZE: MessageSize, S: ApexSamplingPortP4Ext> Borrow<SamplingPortDestination<S>>
        for ConstSamplingPortDestination<MSG_SIZE, S>
    {
        fn borrow(&self) -> &SamplingPortDestination<S> {
            &self.0
        }
    }

    impl<const MSG_SIZE: MessageSize, S: ApexSamplingPortP4Ext> TryFrom<SamplingPortDestination<S>>
        for ConstSamplingPortDestination<MSG_SIZE, S>
    {
        type Error = Error;

        fn try_from(port: SamplingPortDestination<S>) -> Result<Self, Self::Error> {
            if port.msg_size != MSG_SIZE {
                return Err(Error::InvalidConfig);
            }

            Ok(ConstSamplingPortDestination(port))
        }
    }

    #[derive(Debug)]
    pub struct SamplingPortSource<S: ApexSamplingPortP4Ext> {
        _b: PhantomData<AtomicPtr<S>>,
        id: SamplingPortId,
        msg_size: MessageSize,
    }

    impl<S: ApexSamplingPortP4Ext> Clone for SamplingPortSource<S> {
        fn clone(&self) -> Self {
            Self {
                _b: self._b,
                id: self.id,
                msg_size: self.msg_size,
            }
        }
    }

    impl<const MSG_SIZE: MessageSize, S: ApexSamplingPortP4Ext>
        From<ConstSamplingPortSource<MSG_SIZE, S>> for SamplingPortSource<S>
    {
        fn from(port: ConstSamplingPortSource<MSG_SIZE, S>) -> Self {
            port.0
        }
    }

    #[derive(Debug)]
    pub struct SamplingPortDestination<S: ApexSamplingPortP4Ext> {
        _b: PhantomData<AtomicPtr<S>>,
        id: SamplingPortId,
        msg_size: MessageSize,
        refresh: Duration,
    }

    impl<S: ApexSamplingPortP4Ext> Clone for SamplingPortDestination<S> {
        fn clone(&self) -> Self {
            Self {
                _b: self._b,
                id: self.id,
                msg_size: self.msg_size,
                refresh: self.refresh,
            }
        }
    }

    impl<const MSG_SIZE: MessageSize, S: ApexSamplingPortP4Ext>
        From<ConstSamplingPortDestination<MSG_SIZE, S>> for SamplingPortDestination<S>
    {
        fn from(port: ConstSamplingPortDestination<MSG_SIZE, S>) -> Self {
            port.0
        }
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
        fn get_const_sampling_port_source<const MSG_SIZE: MessageSize>(
            name: Name,
        ) -> Result<ConstSamplingPortSource<MSG_SIZE, Self>, Error>;

        /// Returns Err(Error::InvalidConfig) if sampling port with name does not exists or
        /// if the message size of the found sampling port is different than MSG_SIZE
        fn get_const_sampling_port_destination<const MSG_SIZE: MessageSize>(
            name: Name,
        ) -> Result<ConstSamplingPortDestination<MSG_SIZE, Self>, Error>;

        /// Returns Err(Error::InvalidConfig) if sampling port with name does not exists
        fn get_sampling_port_source(name: Name) -> Result<SamplingPortSource<Self>, Error>;

        /// Returns Err(Error::InvalidConfig) if sampling port with name does not exists
        fn get_sampling_port_destination(
            name: Name,
        ) -> Result<SamplingPortDestination<Self>, Error>;
    }

    impl<S: ApexSamplingPortP4> ApexSamplingPortP4Ext for S {
        fn sampling_port_send_unchecked(
            id: SamplingPortId,
            buffer: &[ApexByte],
        ) -> Result<(), Error> {
            S::write_sampling_message(id, buffer)?;
            Ok(())
        }

        unsafe fn sampling_port_receive_unchecked(
            id: SamplingPortId,
            buffer: &mut [ApexByte],
        ) -> Result<(Validity, &[ApexByte]), Error> {
            let (val, len) = S::read_sampling_message(id, buffer)?;
            Ok((val, &buffer[..(len as usize)]))
        }
    }

    impl<S: ApexSamplingPortP1> ApexSamplingPortP1Ext for S {
        /// Returns Err(Error::InvalidConfig) if sampling port with name does not exists or
        /// if the message size of the found sampling port is different than MSG_SIZE
        fn get_const_sampling_port_source<const MSG_SIZE: MessageSize>(
            name: Name,
        ) -> Result<ConstSamplingPortSource<MSG_SIZE, Self>, Error> {
            S::get_sampling_port_source(name)?.try_into()
        }

        /// Returns Err(Error::InvalidConfig) if sampling port with name does not exists or
        /// if the message size of the found sampling port is different than MSG_SIZE
        fn get_const_sampling_port_destination<const MSG_SIZE: MessageSize>(
            name: Name,
        ) -> Result<ConstSamplingPortDestination<MSG_SIZE, Self>, Error> {
            S::get_sampling_port_destination(name)?.try_into()
        }

        /// Returns Err(Error::InvalidConfig) if sampling port with name does not exists
        fn get_sampling_port_source(name: Name) -> Result<SamplingPortSource<Self>, Error> {
            let id = S::get_sampling_port_id(name.into())?;
            // According to ARINC653P1-5 3.6.2.1.5 this can only fail if the sampling_port_id
            //  does not exist in the current partition.
            // But since we retrieve the sampling_port_id directly from the hypervisor
            //  there is no possible way for it not existing
            let SamplingPortStatus {
                refresh_period: _,
                max_message_size: msg_size,
                port_direction,
                ..
            } = S::get_sampling_port_status(id).unwrap().into();

            if port_direction != PortDirection::Source {
                return Err(Error::InvalidConfig);
            }

            Ok(SamplingPortSource {
                _b: Default::default(),
                id,
                msg_size,
            })
        }

        /// Returns Err(Error::InvalidConfig) if sampling port with name does not exists
        fn get_sampling_port_destination(
            name: Name,
        ) -> Result<SamplingPortDestination<Self>, Error> {
            let id = S::get_sampling_port_id(name.into())?;
            // According to ARINC653P1-5 3.6.2.1.5 this can only fail if the sampling_port_id
            //  does not exist in the current partition.
            // But since we retrieve the sampling_port_id directly from the hypervisor
            //  there is no possible way for it not existing
            let SamplingPortStatus {
                refresh_period,
                max_message_size: msg_size,
                port_direction,
                ..
            } = S::get_sampling_port_status(id)?.into();

            if port_direction != PortDirection::Destination {
                return Err(Error::InvalidConfig);
            }

            Ok(SamplingPortDestination {
                _b: Default::default(),
                id,
                msg_size,
                // According to ARINC653P1-5 3.6.2.1.1 the refresh_period defined during
                //  COLD/WARM-Start is always positive, hence this unwrap cannot fail
                refresh: refresh_period.unwrap_duration(),
            })
        }
    }

    impl<S: ApexSamplingPortP4Ext> SamplingPortSource<S> {
        pub fn send(&self, buffer: &[ApexByte]) -> Result<(), Error> {
            buffer.validate_write(self.msg_size)?;
            S::sampling_port_send_unchecked(self.id, buffer)
        }

        pub fn id(&self) -> SamplingPortId {
            self.id
        }

        pub const fn size(&self) -> MessageSize {
            self.msg_size
        }
    }

    impl<S: ApexSamplingPortP1Ext> SamplingPortSource<S> {
        pub fn from_name(name: Name) -> Result<SamplingPortSource<S>, Error> {
            S::get_sampling_port_source(name)
        }

        pub fn status(&self) -> SamplingPortStatus {
            // According to ARINC653P1-5 3.6.2.1.5 this can only fail if the sampling_port_id
            //  does not exist in the current partition.
            // But since we retrieve the sampling_port_id directly from the hypervisor
            //  there is no possible way for it to not exist
            S::get_sampling_port_status(self.id).unwrap().into()
        }
    }

    impl<S: ApexSamplingPortP4Ext> SamplingPortDestination<S> {
        pub fn receive<'a>(
            &self,
            buffer: &'a mut [ApexByte],
        ) -> Result<(Validity, &'a [ApexByte]), Error> {
            buffer.validate_read(self.msg_size)?;
            unsafe { S::sampling_port_receive_unchecked(self.id, buffer) }
        }

        pub fn id(&self) -> SamplingPortId {
            self.id
        }

        pub const fn size(&self) -> MessageSize {
            self.msg_size
        }

        pub fn refresh_period(&self) -> Duration {
            self.refresh
        }
    }

    impl<S: ApexSamplingPortP1Ext> SamplingPortDestination<S> {
        pub fn from_name(name: Name) -> Result<SamplingPortDestination<S>, Error> {
            S::get_sampling_port_destination(name)
        }

        pub fn status(&self) -> SamplingPortStatus {
            // According to ARINC653P1-5 3.6.2.1.5 this can only fail if the sampling_port_id
            //  does not exist in the current partition.
            // But since we retrieve the sampling_port_id directly from the hypervisor
            //  there is no possible way for it not existing
            S::get_sampling_port_status(self.id).unwrap().into()
        }
    }

    impl<S: ApexSamplingPortP4Ext> StartContext<S> {
        pub fn create_const_sampling_port_source<const MSG_SIZE: MessageSize>(
            &mut self,
            name: Name,
        ) -> Result<ConstSamplingPortSource<MSG_SIZE, S>, Error> {
            let port = self.create_sampling_port_source(name, MSG_SIZE)?;
            Ok(ConstSamplingPortSource(port))
        }
        pub fn create_const_sampling_port_destination<const MSG_SIZE: MessageSize>(
            &mut self,
            name: Name,
            refresh: Duration,
        ) -> Result<ConstSamplingPortDestination<MSG_SIZE, S>, Error> {
            let port = self.create_sampling_port_destination(name, MSG_SIZE, refresh)?;
            Ok(ConstSamplingPortDestination(port))
        }

        pub fn create_sampling_port_source(
            &mut self,
            name: Name,
            msg_size: MessageSize,
        ) -> Result<SamplingPortSource<S>, Error> {
            let id = S::create_sampling_port(
                name.into(),
                msg_size,
                PortDirection::Source,
                // use random non-zero duration.
                // while refresh_period is ignored for the source
                //  it may produce an error if set to zero
                SystemTime::Normal(Duration::from_nanos(1)).into(),
            )?;
            Ok(SamplingPortSource {
                _b: Default::default(),
                id,
                msg_size,
            })
        }
        pub fn create_sampling_port_destination(
            &mut self,
            name: Name,
            msg_size: MessageSize,
            refresh: Duration,
        ) -> Result<SamplingPortDestination<S>, Error> {
            let id = S::create_sampling_port(
                name.into(),
                msg_size,
                PortDirection::Destination,
                SystemTime::Normal(refresh).into(),
            )?;
            Ok(SamplingPortDestination {
                _b: Default::default(),
                id,
                msg_size,
                refresh,
            })
        }
    }
}
