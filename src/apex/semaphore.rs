pub mod basic {
    use crate::bindings::*;
    use crate::Locked;

    pub type SemaphoreName = ApexName;

    /// According to ARINC 653P1-5 this may either be 32 or 64 bits.
    /// Internally we will use 64-bit by default.
    /// The implementing Hypervisor may cast this to 32-bit if needed
    pub type SemaphoreId = ApexLongInteger;

    pub type SemaphoreValue = ApexInteger;
    // pub type SemaphoreValueType = ApexInteger;
    // #[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
    // #[cfg_attr(feature = "serde", derive(serde::Serialize))]
    // pub struct SemaphoreValue(SemaphoreValueType);
    // pub const MIN_SEMAPHORE_VALUE: SemaphoreValueType = 0;
    // pub const MAX_SEMAPHORE_VALUE: SemaphoreValueType = 32767;

    // impl TryFrom<SemaphoreValueType> for SemaphoreValue {
    //     type Error = SemaphoreValueType;

    //     fn try_from(value: SemaphoreValueType) -> Result<Self, Self::Error> {
    //         if let MIN_SEMAPHORE_VALUE..=MAX_SEMAPHORE_VALUE = value {
    //             return Ok(SemaphoreValue(value));
    //         }
    //         Err(value)
    //     }
    // }

    // impl From<SemaphoreValue> for SemaphoreValueType {
    //     fn from(sem: SemaphoreValue) -> Self {
    //         sem.0
    //     }
    // }

    // #[cfg(feature = "serde")]
    // impl<'de> serde::Deserialize<'de> for SemaphoreValue {
    //     fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    //     where
    //         D: serde::Deserializer<'de>,
    //     {
    //         let sem: SemaphoreValueType = serde::Deserialize::deserialize(deserializer)?;
    //         sem.try_into().map_err(serde::de::Error::custom)
    //     }
    // }

    #[derive(Debug, Clone, PartialEq, Eq)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
    pub struct SemaphoreStatus {
        pub current_value: SemaphoreValue,
        pub maximum_value: SemaphoreValue,
        pub waiting_processes: WaitingRange,
    }

    pub trait ApexSemaphoreP1 {
        // Only during Warm/Cold-Start
        #[cfg_attr(not(feature = "full_doc"), doc(hidden))]
        fn create_semaphore<L: Locked>(
            semaphore_name: SemaphoreName,
            current_value: SemaphoreValue,
            maximum_value: SemaphoreValue,
            queuing_discipline: QueuingDiscipline,
        ) -> Result<SemaphoreId, ErrorReturnCode>;

        #[cfg_attr(not(feature = "full_doc"), doc(hidden))]
        fn wait_semaphore<L: Locked>(
            semaphore_id: SemaphoreId,
            time_out: ApexSystemTime,
        ) -> Result<(), ErrorReturnCode>;

        #[cfg_attr(not(feature = "full_doc"), doc(hidden))]
        fn signal_semaphore<L: Locked>(semaphore_id: SemaphoreId) -> Result<(), ErrorReturnCode>;

        #[cfg_attr(not(feature = "full_doc"), doc(hidden))]
        fn get_semaphore_id<L: Locked>(
            semaphore_name: SemaphoreName,
        ) -> Result<SemaphoreId, ErrorReturnCode>;

        #[cfg_attr(not(feature = "full_doc"), doc(hidden))]
        fn get_semaphore_status<L: Locked>(
            semaphore_id: SemaphoreId,
        ) -> Result<SemaphoreStatus, ErrorReturnCode>;
    }
}

pub mod abstraction {
    use core::marker::PhantomData;
    use core::sync::atomic::AtomicPtr;

    // Reexport important basic-types for downstream-user
    pub use super::basic::{ApexSemaphoreP1, SemaphoreId, SemaphoreStatus, SemaphoreValue};
    use crate::bindings::*;
    use crate::hidden::Key;
    use crate::prelude::*;

    #[derive(Debug, Clone)]
    pub struct Semaphore<S: ApexSemaphoreP1> {
        _b: PhantomData<AtomicPtr<S>>,
        id: SemaphoreId,
        maximum: SemaphoreValue,
    }

    pub trait ApexSemaphoreP1Ext: ApexSemaphoreP1 + Sized {
        fn get_semaphore(name: Name) -> Result<Semaphore<Self>, Error>;
    }

    impl<S: ApexSemaphoreP1> ApexSemaphoreP1Ext for S {
        fn get_semaphore(name: Name) -> Result<Semaphore<S>, Error> {
            let id = S::get_semaphore_id::<Key>(name.into())?;
            // According to ARINC653P1-5 3.7.2.3.5  this can only fail if the semaphore_id
            //  does not exist in the current partition.
            // But since we retrieve the semaphore_id directly from the hypervisor
            //  there is no possible way for it not existing
            let status = S::get_semaphore_status::<Key>(id).unwrap();

            Ok(Semaphore {
                _b: Default::default(),
                id,
                maximum: status.maximum_value,
            })
        }
    }

    impl<S: ApexSemaphoreP1> Semaphore<S> {
        pub fn from_name(name: Name) -> Result<Semaphore<S>, Error> {
            S::get_semaphore(name)
        }

        pub fn id(&self) -> SemaphoreId {
            self.id
        }

        pub fn maximum(&self) -> SemaphoreValue {
            self.maximum
        }

        pub fn wait(&self, timeout: SystemTime) -> Result<(), Error> {
            S::wait_semaphore::<Key>(self.id, timeout.into())?;
            Ok(())
        }

        pub fn signal(&self) -> Result<(), Error> {
            S::signal_semaphore::<Key>(self.id)?;
            Ok(())
        }

        pub fn current(&self) -> SemaphoreValue {
            self.status().current_value
        }

        pub fn status(&self) -> SemaphoreStatus {
            // According to ARINC653P1-5 3.7.2.3.5  this can only fail if the semaphore_id
            //  does not exist in the current partition.
            // But since we retrieve the semaphore_id directly from the hypervisor
            //  there is no possible way for it not existing
            S::get_semaphore_status::<Key>(self.id).unwrap()
        }
    }

    impl<S: ApexSemaphoreP1> StartContext<S> {
        pub fn create_semaphore(
            &mut self,
            name: Name,
            current: SemaphoreValue,
            maximum: SemaphoreValue,
            qd: QueuingDiscipline,
        ) -> Result<Semaphore<S>, Error> {
            let id = S::create_semaphore::<Key>(name.into(), current, maximum, qd)?;
            Ok(Semaphore {
                _b: Default::default(),
                id,
                maximum,
            })
        }
    }
}
