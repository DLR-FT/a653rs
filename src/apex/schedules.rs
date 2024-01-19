/// bindings for ARINC653P2-4 3.4 schedules
pub mod basic {
    use crate::apex::time::basic::*;
    use crate::apex::types::basic::*;

    pub type ScheduleName = ApexName;

    /// According to ARINC 653P1-5 this may either be 32 or 64 bits.
    /// Internally we will use 64-bit by default.
    /// The implementing Hypervisor may cast this to 32-bit if needed
    pub type ScheduleId = ApexLongInteger;

    pub trait ApexScheduleP2 {
        fn set_module_schedule(schedule_id: ScheduleId) -> Result<(), ErrorReturnCode>;
        fn get_module_schedule_status() -> Result<ApexScheduleStatus, ErrorReturnCode>;
        fn get_module_schedule_id(
            schedule_name: ScheduleName,
        ) -> Result<ScheduleId, ErrorReturnCode>;
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct ApexScheduleStatus {
        pub time_of_last_schedule_switch: ApexSystemTime,
        pub current_schedule: ScheduleId,
        pub next_schedule: ScheduleId,
    }
}

/// abstractions for ARINC653P2-4 3.4 schedules
pub mod abstraction {
    use super::basic::{ApexScheduleP2, ApexScheduleStatus};
    // Reexport important basic-types for downstream-user
    pub use super::basic::{ScheduleId, ScheduleName};
    use crate::prelude::*;

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct ScheduleStatus {
        pub time_of_last_schedule_switch: SystemTime,
        pub current_schedule: ScheduleId,
        pub next_schedule: ScheduleId,
    }

    impl From<ApexScheduleStatus> for ScheduleStatus {
        fn from(s: ApexScheduleStatus) -> Self {
            ScheduleStatus {
                time_of_last_schedule_switch: s.time_of_last_schedule_switch.into(),
                current_schedule: s.current_schedule,
                next_schedule: s.next_schedule,
            }
        }
    }

    pub trait ApexScheduleP2Ext: ApexScheduleP2 + Sized {
        fn set_module_schedule(schedule_id: ScheduleId) -> Result<(), Error>;
        fn get_module_schedule_status() -> Result<ScheduleStatus, Error>;
        fn get_module_schedule_id(schedule_name: ScheduleName) -> Result<ScheduleId, Error>;
    }

    impl<T: ApexScheduleP2> ApexScheduleP2Ext for T {
        fn set_module_schedule(schedule_id: ScheduleId) -> Result<(), Error> {
            T::set_module_schedule(schedule_id)?;
            Ok(())
        }

        fn get_module_schedule_status() -> Result<ScheduleStatus, Error> {
            Ok(T::get_module_schedule_status().map(ScheduleStatus::from)?)
        }

        fn get_module_schedule_id(schedule_name: ScheduleName) -> Result<ScheduleId, Error> {
            Ok(T::get_module_schedule_id(schedule_name)?)
        }
    }
}
