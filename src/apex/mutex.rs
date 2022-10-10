pub mod basic {
    use crate::bindings::*;
    use crate::Locked;
    pub type MutexName = ApexName;

    /// According to ARINC 653P1-5 this may either be 32 or 64 bits.
    /// Internally we will use 64-bit by default.
    /// The implementing Hypervisor may cast this to 32-bit if needed
    pub type MutexId = ApexLongInteger;
    pub type LockCount = ApexInteger;

    pub const NO_MUTEX_OWNED: MutexId = -2;
    pub const PREEMPTION_LOCK_MUTEX: MutexId = -3;

    #[repr(u32)]
    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
    #[cfg_attr(feature = "strum", derive(strum::FromRepr))]
    pub enum MutexState {
        Available = 0,
        Owned = 1,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
    pub struct MutexStatus {
        pub mutex_owner: ProcessId,
        pub mutex_state: MutexState,
        pub mutex_priority: Priority,
        pub lock_count: LockCount,
        pub waiting_processes: WaitingRange,
    }

    /// [`ApexMutexP1`] requires the implementation of the [`ApexProcessP4`] trait
    ///  because [`ApexMutexP1::get_process_mutex_state`] and [`ApexMutexP1::reset_mutex`]
    ///  take a [`ProcessId`] and hence need working process functionalities
    pub trait ApexMutexP1: ApexProcessP4 {
        // Only during Warm/Cold-Start
        #[cfg_attr(not(feature = "full_doc"), doc(hidden))]
        fn create_mutex<L: Locked>(
            mutex_name: MutexName,
            mutex_priority: Priority,
            queuing_discipline: QueuingDiscipline,
        ) -> Result<MutexId, ErrorReturnCode>;

        #[cfg_attr(not(feature = "full_doc"), doc(hidden))]
        fn acquire_mutex<L: Locked>(
            mutex_id: MutexId,
            time_out: ApexSystemTime,
        ) -> Result<(), ErrorReturnCode>;

        #[cfg_attr(not(feature = "full_doc"), doc(hidden))]
        fn release_mutex<L: Locked>(mutex_id: MutexId) -> Result<(), ErrorReturnCode>;

        #[cfg_attr(not(feature = "full_doc"), doc(hidden))]
        fn reset_mutex<L: Locked>(
            mutex_id: MutexId,
            process_id: ProcessId,
        ) -> Result<(), ErrorReturnCode>;

        #[cfg_attr(not(feature = "full_doc"), doc(hidden))]
        fn get_mutex_id<L: Locked>(mutex_name: MutexName) -> Result<MutexId, ErrorReturnCode>;

        #[cfg_attr(not(feature = "full_doc"), doc(hidden))]
        fn get_mutex_status<L: Locked>(mutex_id: MutexId) -> Result<MutexStatus, ErrorReturnCode>;

        #[cfg_attr(not(feature = "full_doc"), doc(hidden))]
        fn get_process_mutex_state<L: Locked>(
            process_id: ProcessId,
        ) -> Result<MutexId, ErrorReturnCode>;
    }
}

pub mod abstraction {
    use core::marker::PhantomData;

    // Reexport important basic-types for downstream-user
    pub use super::basic::{ApexMutexP1, LockCount, MutexId, MutexName, MutexStatus};
    use crate::bindings::*;
    use crate::hidden::Key;
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

    #[derive(Debug, Clone)]
    pub struct Mutex<M: ApexMutexP1> {
        _b: PhantomData<M>,
        id: MutexId,
        priority: Priority,
    }

    pub trait ApexMutexP1Ext: ApexMutexP1 + Sized {
        fn get_mutex(name: Name) -> Result<Mutex<Self>, Error>;
    }

    impl<M: ApexMutexP1> ApexMutexP1Ext for M {
        fn get_mutex(name: Name) -> Result<Mutex<M>, Error> {
            let id = M::get_mutex_id::<Key>(name.into())?;
            // According to ARINC653P1-5 3.7.2.5.6 this can only fail if the mutex_id
            //  does not exist in the current partition.
            // But since we retrieve the mutex_id directly from the hypervisor
            //  there is no possible way for it not existing
            let status = M::get_mutex_status::<Key>(id).unwrap();

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
            M::acquire_mutex::<Key>(self.id, timeout.into())?;
            Ok(())
        }

        pub fn release(&self) -> Result<(), Error> {
            M::release_mutex::<Key>(self.id)?;
            Ok(())
        }

        pub fn reset(&self, process: &Process<M>) -> Result<(), Error> {
            M::reset_mutex::<Key>(self.id, process.id())?;
            Ok(())
        }

        pub fn status(&self) -> MutexStatus {
            // According to ARINC653P1-5 3.7.2.5.6 this can only fail if the mutex_id
            //  does not exist in the current partition.
            // But since we retrieve the mutex_id directly from the hypervisor
            //  there is no possible way for it not existing
            M::get_mutex_status::<Key>(self.id).unwrap()
        }
    }

    impl<A: ApexMutexP1 + ApexProcessP4> Process<A> {
        pub fn get_process_mutex_state(&self) -> MutexOwnedStatus {
            // According to ARINC653P1-5 3.7.2.5.7 this can only fail if the process_id
            //  does not exist in the current partition.
            // But since we retrieve the process_id directly from the hypervisor
            //  there is no possible way for it not existing
            A::get_process_mutex_state::<Key>(self.id()).unwrap().into()
        }
    }

    impl<M: ApexMutexP1> StartContext<M> {
        pub fn create_mutex(
            &mut self,
            name: Name,
            priority: Priority,
            qd: QueuingDiscipline,
        ) -> Result<Mutex<M>, Error> {
            let id = M::create_mutex::<Key>(name.into(), priority, qd)?;
            Ok(Mutex {
                _b: Default::default(),
                id,
                priority,
            })
        }
    }
}
