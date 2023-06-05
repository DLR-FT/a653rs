/// bindings for ARINC653P1-5 3.7.2.4 events
pub mod basic {
    use crate::bindings::*;
    use crate::Locked;

    /// ARINC653P1-5 3.7.1
    pub type EventName = ApexName;

    /// ARINC653P1-5 3.7.1
    ///
    /// According to ARINC 653P1-5 this may either be 32 or 64 bits.
    /// Internally we will use 64-bit by default.
    /// The implementing Hypervisor may cast this to 32-bit if needed
    pub type EventId = ApexLongInteger;

    /// ARINC653P1-5 3.7.1
    #[repr(u32)]
    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
    #[cfg_attr(feature = "strum", derive(strum::FromRepr))]
    pub enum EventState {
        /// inactive
        Down = 0,
        /// active
        Up = 1,
    }

    /// ARINC653P1-5 3.7.1
    #[derive(Debug, Clone, PartialEq, Eq)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
    pub struct EventStatus {
        pub event_state: EventState,
        pub waiting_processes: WaitingRange,
    }

    /// ARINC653P1-5 3.7.2.4 required functions for event functionality
    pub trait ApexEventP1 {
        /// APEX653P1-5 3.7.2.4.1
        ///
        /// # Errors
        /// - [ErrorReturnCode::InvalidConfig]: not enough memory is available
        /// - [ErrorReturnCode::InvalidConfig]: [ApexLimits::SYSTEM_LIMIT_NUMBER_OF_EVENTS](crate::bindings::ApexLimits::SYSTEM_LIMIT_NUMBER_OF_EVENTS) was reached
        /// - [ErrorReturnCode::NoAction]: an event with given `event_name` already exists
        /// - [ErrorReturnCode::InvalidMode]: our current operating mode is [OperatingMode::Normal](crate::prelude::OperatingMode::Normal)
        #[cfg_attr(not(feature = "full_doc"), doc(hidden))]
        fn create_event<L: Locked>(event_name: EventName) -> Result<EventId, ErrorReturnCode>;

        /// APEX653P1-5 3.7.2.4.2 changes events state to [EventState::Up]
        ///
        /// # Errors
        /// - [ErrorReturnCode::InvalidParam]: event with `event_id` does not exist
        #[cfg_attr(not(feature = "full_doc"), doc(hidden))]
        fn set_event<L: Locked>(event_id: EventId) -> Result<(), ErrorReturnCode>;

        /// APEX653P1-5 3.7.2.4.3 changes events state to [EventState::Down]
        ///
        /// # Errors
        /// - [ErrorReturnCode::InvalidParam]: event with `event_id` does not exist
        #[cfg_attr(not(feature = "full_doc"), doc(hidden))]
        fn reset_event<L: Locked>(event_id: EventId) -> Result<(), ErrorReturnCode>;

        /// APEX653P1-5 3.7.2.4.4
        ///
        /// # Errors
        /// - [ErrorReturnCode::InvalidParam]: event with `event_id` does not exist
        /// - [ErrorReturnCode::InvalidParam]: `time_out` is invalid
        /// - [ErrorReturnCode::InvalidMode]: current process holds a mutex
        /// - [ErrorReturnCode::InvalidMode]: current process is error handler AND `time_out` is not instant.
        /// - [ErrorReturnCode::NotAvailable]: `time_out` is instant AND event is [EventState::Down]
        /// - [ErrorReturnCode::TimedOut]: `time_out` elapsed
        #[cfg_attr(not(feature = "full_doc"), doc(hidden))]
        fn wait_event<L: Locked>(
            event_id: EventId,
            time_out: ApexSystemTime,
        ) -> Result<(), ErrorReturnCode>;

        /// APEX653P1-5 3.7.2.4.5
        ///
        /// # Errors
        /// - [ErrorReturnCode::InvalidConfig]: event with `event_name` does not exist
        #[cfg_attr(not(feature = "full_doc"), doc(hidden))]
        fn get_event_id<L: Locked>(event_name: EventName) -> Result<EventId, ErrorReturnCode>;

        /// APEX653P1-5 3.7.2.4.6
        ///
        /// # Errors
        /// - [ErrorReturnCode::InvalidParam]: event with `event_id` does not exist
        #[cfg_attr(not(feature = "full_doc"), doc(hidden))]
        fn get_event_status<L: Locked>(event_id: EventId) -> Result<EventStatus, ErrorReturnCode>;
    }
}

/// abstraction for ARINC653P1-5 3.7.2.4 events
pub mod abstraction {
    use core::marker::PhantomData;
    use core::sync::atomic::AtomicPtr;

    // Reexport important basic-types for downstream-user
    pub use super::basic::{ApexEventP1, EventId, EventState, EventStatus};
    use crate::hidden::Key;
    use crate::prelude::*;

    /// Event abstraction struct
    #[derive(Debug, Clone)]
    pub struct Event<E: ApexEventP1> {
        _b: PhantomData<AtomicPtr<E>>,
        id: EventId,
    }

    /// Free extra functions for implementer of [ApexEventP1]
    pub trait ApexEventP1Ext: ApexEventP1 + Sized {
        /// # Errors
        /// - [Error::InvalidConfig]: event with `name` does not exist
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
        /// # Errors
        /// - [Error::InvalidConfig]: event with `name` does not exist
        pub fn from_name(name: Name) -> Result<Event<E>, Error> {
            E::get_event(name)
        }

        pub fn id(&self) -> EventId {
            self.id
        }

        /// Change to [EventState::Up]
        ///
        /// # Panics
        /// if this event does not exist anymore
        pub fn set(&self) {
            // According to ARINC653P1-5 3.7.2.4.2 this can only fail if the event_id
            //  does not exist in the current partition.
            // But since we retrieve the event_id directly from the hypervisor
            //  there is no possible way for it not existing
            E::set_event::<Key>(self.id).unwrap();
        }

        /// Change to [EventState::Down]
        ///
        /// # Panics
        /// if this event does not exist anymore
        pub fn reset(&self) {
            // According to ARINC653P1-5 3.7.2.4.3 this can only fail if the event_id
            //  does not exist in the current partition.
            // But since we retrieve the event_id directly from the hypervisor
            //  there is no possible way for it not existing
            E::reset_event::<Key>(self.id).unwrap();
        }

        /// wait for this event to occur
        ///
        /// # Errors
        /// - [Error::InvalidParam]: `timeout` is invalid
        /// - [Error::InvalidMode]: current process holds a mutex
        /// - [Error::InvalidMode]: current process is error handler AND `timeout` is not instant.
        /// - [Error::NotAvailable]: `timeout` is instant AND event is [EventState::Down]
        /// - [Error::TimedOut]: `timeout` elapsed
        pub fn wait(&self, timeout: SystemTime) -> Result<(), Error> {
            E::wait_event::<Key>(self.id, timeout.into())?;
            Ok(())
        }

        /// get current event status
        ///
        /// # Panics
        /// if this event does not exist anymore
        pub fn status(&self) -> EventStatus {
            // According to ARINC653P1-5 3.7.2.4.6 this can only fail if the event_id
            //  does not exist in the current partition.
            // But since we retrieve the event_id directly from the hypervisor
            //  there is no possible way for it not existing
            E::get_event_status::<Key>(self.id).unwrap()
        }
    }

    impl<E: ApexEventP1> StartContext<E> {
        /// # Errors
        /// - [Error::InvalidConfig]: not enough memory is available
        /// - [Error::InvalidConfig]: [ApexLimits::SYSTEM_LIMIT_NUMBER_OF_EVENTS](crate::bindings::ApexLimits::SYSTEM_LIMIT_NUMBER_OF_EVENTS) was reached
        /// - [Error::NoAction]: an event with given `name` already exists
        pub fn create_event(&mut self, name: Name) -> Result<Event<E>, Error> {
            let id = E::create_event::<Key>(name.into())?;
            Ok(Event {
                _b: Default::default(),
                id,
            })
        }
    }
}
