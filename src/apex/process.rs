/// bindings for ARINC653P1-5 3.3 process
pub mod basic {
    use crate::apex::time::basic::*;
    use crate::apex::types::basic::*;

    /// ARINC653P1-5 3.3.1
    pub type ProcessName = ApexName;
    /// ARINC653P1-5 3.3.1
    pub type ProcessIndex = ApexInteger;
    /// ARINC653P1-5 3.3.1
    pub type StackSize = ApexUnsigned;

    /// ARINC653P1-5 3.3.1
    pub type Priority = ApexInteger;
    pub const MIN_PRIORITY_VALUE: Priority = 1;
    pub const MAX_PRIORITY_VALUE: Priority = 239;

    /// ARINC653P1-5 3.3.1
    pub type LockLevel = ApexInteger;
    pub const MIN_LOCK_LEVEL: LockLevel = 0;
    pub const MAX_LOCK_LEVEL: LockLevel = 16;

    /// ARINC653P1-5 3.3.1 C compatible function type
    pub type SystemAddress = extern "C" fn();

    /// ARINC653P1-5 3.3.1
    ///
    /// According to ARINC 653P1-5 this may either be 32 or 64 bits.
    /// Internally we will use 64-bit by default.
    /// The implementing Hypervisor may cast this to 32-bit if needed
    pub type ProcessId = ApexLongInteger;
    pub const NULL_PROCESS_ID: ProcessId = 0;
    pub const MAIN_PROCESS_ID: ProcessId = -1;

    /// ARINC653P1-5 3.3.1
    #[repr(u32)]
    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
    #[cfg_attr(feature = "strum", derive(strum::FromRepr))]
    pub enum ProcessState {
        Dormant = 0,
        Ready = 1,
        Running = 2,
        Waiting = 3,
        Faulted = 4,
    }

    /// ARINC653P1-5 3.3.1
    #[repr(u32)]
    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
    #[cfg_attr(feature = "strum", derive(strum::FromRepr))]
    pub enum Deadline {
        Soft = 0,
        Hard = 1,
    }

    /// ARINC653P1-5 3.3.1
    #[repr(C)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct ApexProcessAttribute {
        pub period: ApexSystemTime,
        pub time_capacity: ApexSystemTime,
        pub entry_point: SystemAddress,
        pub stack_size: StackSize,
        pub base_priority: Priority,
        pub deadline: Deadline,
        pub name: ProcessName,
    }

    /// ARINC653P1-5 3.3.1
    #[repr(C)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct ApexProcessStatus {
        pub deadline_time: ApexSystemTime,
        pub current_priority: Priority,
        pub process_state: ProcessState,
        pub attributes: ApexProcessAttribute,
    }

    /// ARINC653P4 3.3.2 required functions for process functionality
    pub trait ApexProcessP4 {
        /// ARINC653P4 3.3.2.3
        ///
        /// # Errors
        /// - [ErrorReturnCode::InvalidConfig]: not enough memory is available
        /// - [ErrorReturnCode::InvalidConfig]: [ApexLimits::SYSTEM_LIMIT_NUMBER_OF_PROCESSES](crate::apex::limits::ApexLimits::SYSTEM_LIMIT_NUMBER_OF_PROCESSES) was reached
        /// - [ErrorReturnCode::NoAction]: a process with given `attributes.name` already exists
        /// - [ErrorReturnCode::InvalidParam]: `attributes.stack_size` is invalid
        /// - [ErrorReturnCode::InvalidParam]: `attributes.base_priority` is invalid
        /// - [ErrorReturnCode::InvalidParam]: `attributes.period` is invalid
        /// - [ErrorReturnCode::InvalidConfig]: `attributes.period` is positive and `attributes.period` is not dividable by the partition period
        /// - [ErrorReturnCode::InvalidParam]: `attributes.time_capacity` is invalid
        /// - [ErrorReturnCode::InvalidParam]: `attributes.period` is positive and `attributes.period` is less than `attributes.time_capacity`
        /// - [ErrorReturnCode::InvalidMode]: our current operating mode is [OperatingMode::Normal](crate::prelude::OperatingMode::Normal)
        fn create_process(attributes: &ApexProcessAttribute) -> Result<ProcessId, ErrorReturnCode>;

        fn start(process_id: ProcessId) -> Result<(), ErrorReturnCode>;
    }

    pub trait ApexProcessP1: ApexProcessP4 {
        fn set_priority(process_id: ProcessId, priority: Priority) -> Result<(), ErrorReturnCode>;

        fn suspend_self(time_out: ApexSystemTime) -> Result<(), ErrorReturnCode>;

        fn suspend(process_id: ProcessId) -> Result<(), ErrorReturnCode>;

        fn resume(process_id: ProcessId) -> Result<(), ErrorReturnCode>;

        fn stop_self();

        fn stop(process_id: ProcessId) -> Result<(), ErrorReturnCode>;

        fn delayed_start(
            process_id: ProcessId,
            delay_time: ApexSystemTime,
        ) -> Result<(), ErrorReturnCode>;

        fn lock_preemption() -> Result<LockLevel, ErrorReturnCode>;

        fn unlock_preemption() -> Result<LockLevel, ErrorReturnCode>;

        fn get_my_id() -> Result<ProcessId, ErrorReturnCode>;

        fn get_process_id(process_name: ProcessName) -> Result<ProcessId, ErrorReturnCode>;

        fn get_process_status(process_id: ProcessId) -> Result<ApexProcessStatus, ErrorReturnCode>;

        // Only during Warm/Cold-Start
        fn initialize_process_core_affinity(
            process_id: ProcessId,
            processor_core_id: ProcessorCoreId,
        ) -> Result<(), ErrorReturnCode>;

        fn get_my_processor_core_id() -> ProcessorCoreId;

        fn get_my_index() -> Result<ProcessIndex, ErrorReturnCode>;
    }
}

/// abstractions for ARINC653P1-5 3.3 process
pub mod abstraction {
    use core::marker::PhantomData;
    use core::sync::atomic::AtomicPtr;

    use super::basic::{ApexProcessAttribute, ApexProcessP1, ApexProcessP4, ApexProcessStatus};
    // Reexport important basic-types for downstream-user
    pub use super::basic::{
        Deadline, LockLevel, Priority, ProcessId, ProcessIndex, ProcessName, StackSize,
        SystemAddress, MAIN_PROCESS_ID, MAX_LOCK_LEVEL, MAX_PRIORITY_VALUE, MIN_LOCK_LEVEL,
        MIN_PRIORITY_VALUE, NULL_PROCESS_ID,
    };
    use crate::prelude::*;

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct ProcessAttribute {
        pub period: SystemTime,
        pub time_capacity: SystemTime,
        pub entry_point: SystemAddress,
        pub stack_size: StackSize,
        pub base_priority: Priority,
        pub deadline: Deadline,
        pub name: Name,
    }

    impl From<ProcessAttribute> for ApexProcessAttribute {
        fn from(p: ProcessAttribute) -> Self {
            ApexProcessAttribute {
                period: p.period.into(),
                time_capacity: p.time_capacity.into(),
                entry_point: p.entry_point,
                stack_size: p.stack_size,
                base_priority: p.base_priority,
                deadline: p.deadline,
                name: p.name.into(),
            }
        }
    }

    impl From<ApexProcessAttribute> for ProcessAttribute {
        fn from(p: ApexProcessAttribute) -> Self {
            ProcessAttribute {
                period: p.period.into(),
                time_capacity: p.time_capacity.into(),
                entry_point: p.entry_point,
                stack_size: p.stack_size,
                base_priority: p.base_priority,
                deadline: p.deadline,
                name: Name::new(p.name),
            }
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct ProcessStatus {
        pub deadline_time: SystemTime,
        pub current_priority: Priority,
        pub process_state: super::basic::ProcessState,
        pub attributes: ProcessAttribute,
    }

    impl From<ApexProcessStatus> for ProcessStatus {
        fn from(p: ApexProcessStatus) -> Self {
            ProcessStatus {
                deadline_time: p.deadline_time.into(),
                current_priority: p.current_priority,
                process_state: p.process_state,
                attributes: p.attributes.into(),
            }
        }
    }

    #[derive(Debug)]
    pub struct Process<P: ApexProcessP4> {
        _p: PhantomData<AtomicPtr<P>>,
        id: ProcessId,
    }

    impl<P: ApexProcessP4> Clone for Process<P> {
        fn clone(&self) -> Self {
            Self {
                _p: self._p,
                id: self.id,
            }
        }
    }

    pub trait ApexProcessP1Ext: ApexProcessP1 + Sized {
        fn get_process(name: Name) -> Result<Process<Self>, Error>;
    }

    impl<P: ApexProcessP1> ApexProcessP1Ext for P {
        fn get_process(name: Name) -> Result<Process<P>, Error> {
            let id = P::get_process_id(name.into())?;
            Ok(Process {
                _p: Default::default(),
                id,
            })
        }
    }

    impl<P: ApexProcessP4> Process<P> {
        pub fn start(&self) -> Result<(), Error> {
            P::start(self.id)?;
            Ok(())
        }

        pub fn id(&self) -> ProcessId {
            self.id
        }
    }

    impl<P: ApexProcessP1> Process<P> {
        pub fn from_name(name: Name) -> Result<Process<P>, Error> {
            P::get_process(name)
        }

        pub fn get_self() -> Result<Process<P>, Error> {
            let id = P::get_my_id()?;
            Ok(Process {
                _p: Default::default(),
                id,
            })
        }

        pub fn set_priority(&self, priority: Priority) -> Result<(), Error> {
            P::set_priority(self.id, priority)?;
            Ok(())
        }

        pub fn suspend_self(time_out: SystemTime) -> Result<(), Error> {
            P::suspend_self(time_out.into())?;
            Ok(())
        }

        pub fn suspend(&self) -> Result<(), Error> {
            P::suspend(self.id)?;
            Ok(())
        }

        pub fn resume(&self) -> Result<(), Error> {
            P::resume(self.id)?;
            Ok(())
        }

        pub fn stop_self() {
            P::stop_self()
        }

        pub fn stop(&self) -> Result<(), Error> {
            P::stop(self.id)?;
            Ok(())
        }

        pub fn delayed_start(&self, delay_time: SystemTime) -> Result<(), Error> {
            P::delayed_start(self.id, delay_time.into())?;
            Ok(())
        }

        pub fn lock_preemption() -> Result<LockLevel, Error> {
            Ok(P::lock_preemption()?)
        }

        pub fn unlock_preemption() -> Result<LockLevel, Error> {
            Ok(P::unlock_preemption()?)
        }

        pub fn status(&self) -> ProcessStatus {
            // According to ARINC653P1-5 3.3.2.2 this can only fail if the processId
            //  does not exist in the current partition.
            // But since we retrieve the processId directly from the hypervisor
            //  there is no possible way for it not existing
            P::get_process_status(self.id).unwrap().into()
        }

        pub fn get_my_processor_core_id() -> ProcessorCoreId {
            P::get_my_processor_core_id()
        }

        pub fn get_my_index() -> Result<ProcessIndex, Error> {
            Ok(P::get_my_index()?)
        }
    }

    impl<P: ApexProcessP4> StartContext<P> {
        pub fn create_process(&mut self, attr: ProcessAttribute) -> Result<Process<P>, Error> {
            let id = P::create_process(&attr.into())?;
            Ok(Process {
                _p: Default::default(),
                id,
            })
        }
    }

    impl<P: ApexProcessP1> StartContext<P> {
        pub fn initialize_process_core_affinity(
            &self,
            process: &Process<P>,
            processor_core_id: ProcessorCoreId,
        ) -> Result<(), Error> {
            P::initialize_process_core_affinity(process.id, processor_core_id)?;
            Ok(())
        }
    }
}
