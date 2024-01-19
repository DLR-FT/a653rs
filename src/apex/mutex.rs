/// bindings for ARINC653P1-5 3.7.2.5 mutex
pub mod basic {
    use crate::apex::process::basic::*;
    use crate::apex::time::basic::*;
    use crate::apex::types::basic::*;

    /// ARINC653P1-5 3.7.1
    pub type MutexName = ApexName;

    /// ARINC653P1-5 3.7.1
    ///
    /// According to ARINC 653P1-5 this may either be 32 or 64 bits.
    /// Internally we will use 64-bit by default.
    /// The implementing Hypervisor may cast this to 32-bit if needed
    pub type MutexId = ApexLongInteger;

    /// ARINC653P1-5 3.7.1
    pub type LockCount = ApexInteger;

    /// ARINC653P1-5 3.7.2.5
    pub const NO_MUTEX_OWNED: MutexId = -2;
    /// ARINC653P1-5 3.7.2.5
    pub const PREEMPTION_LOCK_MUTEX: MutexId = -3;

    /// ARINC653P1-5 3.7.1
    #[repr(u32)]
    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
    #[cfg_attr(feature = "strum", derive(strum::FromRepr))]
    pub enum MutexState {
        Available = 0,
        Owned = 1,
    }

    /// ARINC653P1-5 3.7.1
    #[derive(Debug, Clone, PartialEq, Eq)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
    pub struct MutexStatus {
        pub mutex_owner: ProcessId,
        pub mutex_state: MutexState,
        pub mutex_priority: Priority,
        pub lock_count: LockCount,
        pub waiting_processes: WaitingRange,
    }

    /// ARINC653P1-5 required functions for Mutex functionality
    ///
    /// [`ApexMutexP1`] requires the implementation of the [`ApexProcessP4`] trait
    ///  because [`ApexMutexP1::get_process_mutex_state`] and [`ApexMutexP1::reset_mutex`]
    ///  take a [`ProcessId`] and hence need working process functionalities
    pub trait ApexMutexP1: ApexProcessP4 {
        /// # Errors
        /// - [ErrorReturnCode::InvalidConfig]: [ApexLimits::SYSTEM_LIMIT_NUMBER_OF_MUTEXES](crate::apex::limits::ApexLimits::SYSTEM_LIMIT_NUMBER_OF_MUTEXES) was reached
        /// - [ErrorReturnCode::NoAction]: an mutex with given `mutex_name` already exists in this partition
        /// - [ErrorReturnCode::InvalidParam]: `mutex_priority` is invalid
        /// - [ErrorReturnCode::InvalidParam]: [QueuingDiscipline](crate::apex::types::basic::QueuingDiscipline) in `queuing_discipline` is unsupported
        /// - [ErrorReturnCode::InvalidMode]: our current operating mode is [OperatingMode::Normal](crate::prelude::OperatingMode::Normal)
        #[cfg_attr(not(feature = "full_doc"), doc(hidden))]
        fn create_mutex(
            mutex_name: MutexName,
            mutex_priority: Priority,
            queuing_discipline: QueuingDiscipline,
        ) -> Result<MutexId, ErrorReturnCode>;

        /// # Errors
        /// - [ErrorReturnCode::InvalidParam]: mutex with given `mutex_id` does not exist in this partition
        /// - [ErrorReturnCode::InvalidParam]: `mutex_id` is [PREEMPTION_LOCK_MUTEX]
        /// - [ErrorReturnCode::InvalidParam]: `time_out` is invalid
        /// - [ErrorReturnCode::InvalidMode]: different mutex is already held by this process
        /// - [ErrorReturnCode::InvalidMode]: this process is the error handler
        /// - [ErrorReturnCode::InvalidMode]: the priority of this process is greater than the priority of the given mutex
        /// - [ErrorReturnCode::NotAvailable]:
        /// - [ErrorReturnCode::TimedOut]: `time_out` elapsed
        /// - [ErrorReturnCode::InvalidConfig]: lock count of given mutex is at [MAX_LOCK_LEVEL](crate::apex::process::basic::MAX_LOCK_LEVEL)
        #[cfg_attr(not(feature = "full_doc"), doc(hidden))]
        fn acquire_mutex(
            mutex_id: MutexId,
            time_out: ApexSystemTime,
        ) -> Result<(), ErrorReturnCode>;

        #[cfg_attr(not(feature = "full_doc"), doc(hidden))]
        fn release_mutex(mutex_id: MutexId) -> Result<(), ErrorReturnCode>;

        #[cfg_attr(not(feature = "full_doc"), doc(hidden))]
        fn reset_mutex(mutex_id: MutexId, process_id: ProcessId) -> Result<(), ErrorReturnCode>;

        #[cfg_attr(not(feature = "full_doc"), doc(hidden))]
        fn get_mutex_id(mutex_name: MutexName) -> Result<MutexId, ErrorReturnCode>;

        #[cfg_attr(not(feature = "full_doc"), doc(hidden))]
        fn get_mutex_status(mutex_id: MutexId) -> Result<MutexStatus, ErrorReturnCode>;

        #[cfg_attr(not(feature = "full_doc"), doc(hidden))]
        fn get_process_mutex_state(process_id: ProcessId) -> Result<MutexId, ErrorReturnCode>;
    }
}

/// abstractions for ARINC653P1-5 3.7.2.5 mutex
pub mod abstraction {
    use core::marker::PhantomData;
    use core::sync::atomic::AtomicPtr;

    use super::basic::{ApexMutexP1, NO_MUTEX_OWNED, PREEMPTION_LOCK_MUTEX};
    // Reexport important basic-types for downstream-user
    pub use super::basic::{LockCount, MutexId, MutexName, MutexStatus};
    use crate::apex::process::basic::ApexProcessP4;
    use crate::prelude::*;

    #[derive(Debug, Clone, PartialEq, Eq)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
    pub enum MutexOwnedStatus {
        Owned(MutexId),
        NoMutexOwned,
        PreemptionLockMutex,
    }

    impl From<MutexId> for MutexOwnedStatus {
        fn from(value: MutexId) -> Self {
            use MutexOwnedStatus::*;
            match value {
                NO_MUTEX_OWNED => NoMutexOwned,
                PREEMPTION_LOCK_MUTEX => PreemptionLockMutex,
                id => Owned(id),
            }
        }
    }

    #[derive(Debug)]
    pub struct Mutex<M: ApexMutexP1> {
        _b: PhantomData<AtomicPtr<M>>,
        id: MutexId,
        priority: Priority,
    }

    impl<M: ApexMutexP1> Clone for Mutex<M> {
        fn clone(&self) -> Self {
            Self {
                _b: self._b,
                id: self.id,
                priority: self.priority,
            }
        }
    }

    pub trait ApexMutexP1Ext: ApexMutexP1 + Sized {
        fn get_mutex(name: Name) -> Result<Mutex<Self>, Error>;
    }

    impl<M: ApexMutexP1> ApexMutexP1Ext for M {
        fn get_mutex(name: Name) -> Result<Mutex<M>, Error> {
            let id = M::get_mutex_id(name.into())?;
            // According to ARINC653P1-5 3.7.2.5.6 this can only fail if the mutex_id
            //  does not exist in the current partition.
            // But since we retrieve the mutex_id directly from the hypervisor
            //  there is no possible way for it not existing
            let status = M::get_mutex_status(id).unwrap();

            Ok(Mutex {
                _b: Default::default(),
                id,
                priority: status.mutex_priority,
            })
        }
    }

    impl<M: ApexMutexP1> Mutex<M> {
        pub fn from_name(name: Name) -> Result<Mutex<M>, Error> {
            M::get_mutex(name)
        }

        pub fn id(&self) -> MutexId {
            self.id
        }

        pub fn priority(&self) -> Priority {
            self.priority
        }

        pub fn acquire(&self, timeout: SystemTime) -> Result<(), Error> {
            M::acquire_mutex(self.id, timeout.into())?;
            Ok(())
        }

        pub fn release(&self) -> Result<(), Error> {
            M::release_mutex(self.id)?;
            Ok(())
        }

        pub fn reset(&self, process: &Process<M>) -> Result<(), Error> {
            M::reset_mutex(self.id, process.id())?;
            Ok(())
        }

        pub fn status(&self) -> MutexStatus {
            // According to ARINC653P1-5 3.7.2.5.6 this can only fail if the mutex_id
            //  does not exist in the current partition.
            // But since we retrieve the mutex_id directly from the hypervisor
            //  there is no possible way for it not existing
            M::get_mutex_status(self.id).unwrap()
        }
    }

    impl<A: ApexMutexP1 + ApexProcessP4> Process<A> {
        pub fn get_process_mutex_state(&self) -> MutexOwnedStatus {
            // According to ARINC653P1-5 3.7.2.5.7 this can only fail if the process_id
            //  does not exist in the current partition.
            // But since we retrieve the process_id directly from the hypervisor
            //  there is no possible way for it not existing
            A::get_process_mutex_state(self.id()).unwrap().into()
        }
    }

    impl<M: ApexMutexP1> StartContext<M> {
        pub fn create_mutex(
            &mut self,
            name: Name,
            priority: Priority,
            qd: QueuingDiscipline,
        ) -> Result<Mutex<M>, Error> {
            let id = M::create_mutex(name.into(), priority, qd)?;
            Ok(Mutex {
                _b: Default::default(),
                id,
                priority,
            })
        }
    }
}
