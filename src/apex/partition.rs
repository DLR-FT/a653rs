/// bindings for ARINC653P1-5 3.2.2 partition
pub mod basic {
    use crate::apex::process::basic::*;
    use crate::apex::time::basic::*;
    use crate::apex::types::basic::*;

    /// According to ARINC 653P1-5 this may either be 32 or 64 bits.
    /// Internally we will use 64-bit by default.
    /// The implementing Hypervisor may cast this to 32-bit if needed
    pub type PartitionId = ApexLongInteger;
    pub type NumCores = ApexUnsigned;

    #[repr(u32)]
    #[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
    #[cfg_attr(feature = "strum", derive(strum::FromRepr))]
    pub enum OperatingMode {
        #[default]
        Idle = 0,
        ColdStart = 1,
        WarmStart = 2,
        Normal = 3,
    }

    impl TryFrom<ApexUnsigned> for OperatingMode {
        type Error = ApexUnsigned;

        fn try_from(value: ApexUnsigned) -> Result<Self, Self::Error> {
            match value {
                0 => Ok(OperatingMode::Idle),
                1 => Ok(OperatingMode::ColdStart),
                2 => Ok(OperatingMode::WarmStart),
                3 => Ok(OperatingMode::Normal),
                e => Err(e),
            }
        }
    }

    #[repr(u32)]
    #[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
    #[cfg_attr(feature = "strum", derive(strum::FromRepr))]
    pub enum StartCondition {
        #[default]
        NormalStart = 0,
        PartitionRestart = 1,
        HmModuleRestart = 2,
        HmPartitionRestart = 3,
    }

    impl TryFrom<ApexUnsigned> for StartCondition {
        type Error = ApexUnsigned;

        fn try_from(value: ApexUnsigned) -> Result<Self, Self::Error> {
            match value {
                0 => Ok(StartCondition::NormalStart),
                1 => Ok(StartCondition::PartitionRestart),
                2 => Ok(StartCondition::HmModuleRestart),
                3 => Ok(StartCondition::HmPartitionRestart),
                e => Err(e),
            }
        }
    }

    #[repr(C)]
    #[derive(Debug, Clone, PartialEq, Eq, Default)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
    pub struct ApexPartitionStatus {
        pub period: ApexSystemTime,
        pub duration: ApexSystemTime,
        pub identifier: PartitionId,
        pub lock_level: LockLevel,
        pub operating_mode: OperatingMode,
        pub start_condition: StartCondition,
        pub num_assigned_cores: NumCores,
    }

    pub trait ApexPartitionP4 {
        // As stated in ARINC653P1-5 3.2.2.1, this never fails
        fn get_partition_status() -> ApexPartitionStatus;

        fn set_partition_mode(operating_mode: OperatingMode) -> Result<(), ErrorReturnCode>;
    }
}

/// abstraction for ARINC653P1-5 3.2.2 partition
pub mod abstraction {
    use core::marker::PhantomData;
    use core::sync::atomic::AtomicPtr;

    use super::basic::{ApexPartitionP4, ApexPartitionStatus};
    // Reexport important basic-types for downstream-user
    pub use super::basic::{NumCores, OperatingMode, PartitionId, StartCondition};
    use crate::prelude::*;

    #[derive(Debug, Clone, PartialEq, Eq)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
    pub struct PartitionStatus {
        pub period: SystemTime,
        pub duration: SystemTime,
        pub identifier: PartitionId,
        pub lock_level: LockLevel,
        pub operating_mode: OperatingMode,
        pub start_condition: StartCondition,
        pub num_assigned_cores: NumCores,
    }

    impl From<ApexPartitionStatus> for PartitionStatus {
        fn from(s: ApexPartitionStatus) -> Self {
            PartitionStatus {
                period: s.period.into(),
                duration: s.duration.into(),
                identifier: s.identifier,
                lock_level: s.lock_level,
                operating_mode: s.operating_mode,
                start_condition: s.start_condition,
                num_assigned_cores: s.num_assigned_cores,
            }
        }
    }

    #[derive(Debug)]
    pub struct StartContext<A> {
        _a: PhantomData<AtomicPtr<A>>,
    }

    pub trait PartitionExt<A>: Partition<A>
    where
        A: ApexPartitionP4,
    {
        fn get_status() -> PartitionStatus;

        /// change partition mode  
        /// DO NOT CALL THIS WITH [OperatingMode::Normal] FROM THE START FUNCTION.
        ///
        /// # Errors
        /// - [Error::NoAction]: `mode` is [OperatingMode::Normal] and partition mode is [OperatingMode::Normal]
        /// - [Error::InvalidMode]: `mode` is [OperatingMode::WarmStart] and partition mode is [OperatingMode::ColdStart]
        fn set_mode(mode: OperatingMode) -> Result<(), Error>;

        fn run(self) -> !;
    }

    impl<A, P> PartitionExt<A> for P
    where
        P: Partition<A>,
        A: ApexPartitionP4,
    {
        fn get_status() -> PartitionStatus {
            A::get_partition_status().into()
        }

        fn set_mode(mode: OperatingMode) -> Result<(), Error> {
            Ok(A::set_partition_mode(mode)?)
        }

        fn run(self) -> ! {
            let mut ctx = StartContext {
                _a: Default::default(),
            };
            let status = Self::get_status();

            match status.operating_mode {
                OperatingMode::ColdStart => self.cold_start(&mut ctx),
                OperatingMode::WarmStart => self.warm_start(&mut ctx),
                // As per ARINC653P1-5 Figure 2.3.1.4, this can not happen
                unexpected => panic!("{unexpected:?}"),
            };

            // As stated in ARINC653P1-5 3.2.2.2, this can not fail,
            // because we are either in COLD_START or WARM_START
            A::set_partition_mode(OperatingMode::Normal).unwrap();

            #[allow(clippy::empty_loop)]
            loop {
                //Verify
            }
        }
    }

    pub trait Partition<P>: Sized
    where
        P: ApexPartitionP4,
    {
        fn cold_start(&self, ctx: &mut StartContext<P>);
        fn warm_start(&self, ctx: &mut StartContext<P>);
    }
}
