pub mod basic {
    use crate::bindings::*;
    use crate::Locked;

    pub type EventName = ApexName;

    /// According to ARINC 653P1-5 this may either be 32 or 64 bits.
    /// Internally we will use 64-bit by default.
    /// The implementing Hypervisor may cast this to 32-bit if needed
    pub type EventId = ApexLongInteger;

    #[repr(u32)]
    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
    #[cfg_attr(feature = "strum", derive(strum::FromRepr))]
    pub enum EventState {
        Down = 0,
        Up = 1,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
    pub struct EventStatus {
        pub event_state: EventState,
        pub waiting_processes: WaitingRange,
    }

    pub trait ApexEventP1 {
        // Only during Warm/Cold-Start
        fn create_event<L: Locked>(event_name: EventName) -> Result<EventId, ErrorReturnCode>;

        fn set_event<L: Locked>(event_id: EventId) -> Result<(), ErrorReturnCode>;

        fn reset_event<L: Locked>(event_id: EventId) -> Result<(), ErrorReturnCode>;

        fn wait_event<L: Locked>(
            event_id: EventId,
            time_out: ApexSystemTime,
        ) -> Result<(), ErrorReturnCode>;

        fn get_event_id<L: Locked>(event_name: EventName) -> Result<EventId, ErrorReturnCode>;

        fn get_event_status<L: Locked>(event_id: EventId) -> Result<EventStatus, ErrorReturnCode>;
    }
}

pub mod abstraction {
    use core::marker::PhantomData;

    // Reexport important basic-types for downstream-user
    pub use super::basic::{ApexEventP1, EventId, EventState, EventStatus};
    use crate::hidden::Key;
    use crate::prelude::*;

    #[derive(Debug, Clone)]
    pub struct Event<E: ApexEventP1> {
        _b: PhantomData<E>,
        id: EventId,
    }

    pub trait ApexEventP1Ext: ApexEventP1 + Sized {
        fn get_event(name: Name) -> Result<Event<Self>, Error>;
    }

    impl<E: ApexEventP1> ApexEventP1Ext for E {
        fn get_event(name: Name) -> Result<Event<E>, Error> {
            let id = E::get_event_id::<Key>(name.into())?;

            Ok(Event {
                _b: Default::default(),
                id,
            })
        }
    }

    impl<E: ApexEventP1> Event<E> {
        pub fn from_name(name: Name) -> Result<Event<E>, Error> {
            E::get_event(name)
        }

        pub fn id(&self) -> EventId {
            self.id
        }

        pub fn set(&self) {
            // According to ARINC653P1-5 3.7.2.4.2 this can only fail if the event_id
            //  does not exist in the current partition.
            // But since we retrieve the event_id directly from the hypervisor
            //  there is no possible way for it not existing
            E::set_event::<Key>(self.id).unwrap();
        }

        pub fn reset(&self) {
            // According to ARINC653P1-5 3.7.2.4.3 this can only fail if the event_id
            //  does not exist in the current partition.
            // But since we retrieve the event_id directly from the hypervisor
            //  there is no possible way for it not existing
            E::reset_event::<Key>(self.id).unwrap();
        }

        pub fn wait(&self, timeout: SystemTime) -> Result<(), Error> {
            E::wait_event::<Key>(self.id, timeout.into())?;
            Ok(())
        }

        pub fn status(&self) -> EventStatus {
            // According to ARINC653P1-5 3.7.2.4.6 this can only fail if the event_id
            //  does not exist in the current partition.
            // But since we retrieve the event_id directly from the hypervisor
            //  there is no possible way for it not existing
            E::get_event_status::<Key>(self.id).unwrap()
        }
    }

    impl<E: ApexEventP1> StartContext<E> {
        pub fn create_event(&mut self, name: Name) -> Result<Event<E>, Error> {
            let id = E::create_event::<Key>(name.into())?;
            Ok(Event {
                _b: Default::default(),
                id,
            })
        }
    }
}
