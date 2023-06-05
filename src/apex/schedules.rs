/// bindings for ARINC653P2-4 3.4 schedules
pub mod basic {
    use crate::bindings::*;

    pub type ScheduleName = ApexName;

    /// According to ARINC 653P1-5 this may either be 32 or 64 bits.
    /// Internally we will use 64-bit by default.
    /// The implementing Hypervisor may cast this to 32-bit if needed
    pub type ScheduleId = ApexLongInteger;

    pub trait ApexScheduleP2 {
        #[cfg_attr(not(feature = "full_doc"), doc(hidden))]
        fn set_module_schedule<L: Locked>(schedule_id: ScheduleId) -> Result<(), ErrorReturnCode>;
        #[cfg_attr(not(feature = "full_doc"), doc(hidden))]
        fn get_module_schedule_status<L: Locked>() -> Result<ApexScheduleStatus, ErrorReturnCode>;
        #[cfg_attr(not(feature = "full_doc"), doc(hidden))]
        fn get_module_schedule_id<L: Locked>(
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
    use super::basic::{ApexScheduleP2, ApexScheduleStatus, ScheduleId, ScheduleName};
    use crate::hidden::Key;
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
            T::set_module_schedule::<Key>(schedule_id)?;
            Ok(())
        }

        fn get_module_schedule_status() -> Result<ScheduleStatus, Error> {
            Ok(T::get_module_schedule_status::<Key>().map(ScheduleStatus::from)?)
        }

        fn get_module_schedule_id(schedule_name: ScheduleName) -> Result<ScheduleId, Error> {
            Ok(T::get_module_schedule_id::<Key>(schedule_name)?)
        }
    }
}
