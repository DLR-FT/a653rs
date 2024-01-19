/// bindings for ARINC653P1-5 3.4.2 time
pub mod basic {
    use crate::apex::types::basic::*;

    /// ARINC653P1-5 3.4.1
    pub type ApexSystemTime = ApexLongInteger;
    pub const INFINITE_TIME_VALUE: ApexSystemTime = -1;

    /// ARINC653P4 3.4 required functions for time management functionality
    pub trait ApexTimeP4 {
        /// APEX653P4 3.4.2.2 wait until next release
        ///
        /// # Errors
        /// - [ErrorReturnCode::InvalidMode]: calling process is not a periodic process
        /// - (P1/P2 only) [ErrorReturnCode::InvalidMode]: calling process holds a mutex
        /// - (P1/P2 only) [ErrorReturnCode::InvalidMode]: calling process is error handler
        /// - (P1/P2 only) [ErrorReturnCode::InvalidConfig]: deadline calulation failed
        fn periodic_wait() -> Result<(), ErrorReturnCode>;

        /// APEX653P4 3.4.2.3
        fn get_time() -> ApexSystemTime;
    }

    /// ARINC653P1-5 3.4 required functions for time management functionality
    pub trait ApexTimeP1: ApexTimeP4 {
        /// ARINC653P1-5 3.4.2.1
        ///
        /// # Errors
        /// - [ErrorReturnCode::InvalidMode]: calling process holds a mutex
        /// - [ErrorReturnCode::InvalidMode]: calling process is error handler
        /// - [ErrorReturnCode::InvalidParam]: `delay_time` is too large
        /// - [ErrorReturnCode::InvalidParam]: `delay_time` is negative (infinite)
        fn timed_wait(delay_time: ApexSystemTime) -> Result<(), ErrorReturnCode>;

        /// ARINC653P1-5 3.4.2.4 update deadline
        ///
        /// # Errors
        /// - [ErrorReturnCode::InvalidParam]: `budget_time` is invalid
        /// - [ErrorReturnCode::InvalidMode]: calling process is periodic AND calulated deadline exceeds next release point
        /// - [ErrorReturnCode::NoAction]: calling process is error handler
        /// - [ErrorReturnCode::NoAction]: our current operating mode is not [OperatingMode::Normal](crate::prelude::OperatingMode::Normal)
        fn replenish(budget_time: ApexSystemTime) -> Result<(), ErrorReturnCode>;
    }
}

/// abstractions for ARINC653P1-5 3.4.2 time
pub mod abstraction {
    use core::time::Duration;

    use super::basic::{ApexSystemTime, ApexTimeP1, ApexTimeP4, INFINITE_TIME_VALUE};
    use crate::prelude::*;

    /// Abstracted SystemTime Variant making use of Rusts [Duration]
    /// Includes Infinite-variant since [Duration] does not allow for negative values
    ///
    /// # Size
    ///
    /// [ApexSystemTime] => 8-Byte  
    /// [Duration] => 16-Byte  
    /// [SystemTime] => 24-Byte
    #[repr(C)]
    #[derive(Clone, Debug, PartialEq, Eq)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
    pub enum SystemTime {
        Infinite,
        Normal(Duration),
    }

    impl SystemTime {
        pub fn new(time: ApexSystemTime) -> Self {
            time.into()
        }

        /// # Panics
        /// If this SystemTime is [SystemTime::Infinite]
        pub fn unwrap_duration(self) -> Duration {
            if let SystemTime::Normal(time) = self {
                return time;
            }
            panic!("Is infinite")
        }
    }

    impl From<Duration> for SystemTime {
        fn from(time: Duration) -> Self {
            Self::Normal(time)
        }
    }

    impl From<SystemTime> for Option<Duration> {
        fn from(time: SystemTime) -> Self {
            match time {
                SystemTime::Infinite => None,
                SystemTime::Normal(time) => Some(time),
            }
        }
    }

    impl From<Option<Duration>> for SystemTime {
        fn from(time: Option<Duration>) -> Self {
            use SystemTime::*;
            match time {
                Some(time) => Normal(time),
                None => Infinite,
            }
        }
    }

    impl From<ApexSystemTime> for SystemTime {
        /// Converts ApexSystemTime to a [SystemTime]  
        /// Should ApexSystemTime be less than 0, its considered to be infinite.  
        /// As stated in ARINC653P1-5 3.4.1 all negative values should be treated as `INFINITE_TIME_VALUE`
        fn from(time: ApexSystemTime) -> Self {
            use SystemTime::*;
            // This conversion can only fail, if ApexSystemTime is negative.
            match u64::try_from(time) {
                Ok(time) => Normal(Duration::from_nanos(time)),
                Err(_) => Infinite,
            }
        }
    }

    impl From<SystemTime> for ApexSystemTime {
        fn from(time: SystemTime) -> Self {
            if let SystemTime::Normal(time) = time {
                if let Ok(time) = ApexSystemTime::try_from(time.as_nanos()) {
                    return time;
                }
            }
            INFINITE_TIME_VALUE
        }
    }

    /// Free extra functions for implementer of [ApexTimeP4]
    pub trait ApexTimeP4Ext: ApexTimeP4 + Sized {
        /// wait until next release
        ///
        /// # Errors
        /// - [Error::InvalidMode]: calling process is not a periodic process
        /// - (P1/P2 only) [Error::InvalidMode]: calling process holds a mutex
        /// - (P1/P2 only) [Error::InvalidMode]: calling process is error handler
        /// - (P1/P2 only) [Error::InvalidConfig]: deadline calulation failed
        fn periodic_wait() -> Result<(), Error>;

        fn get_time() -> SystemTime;
    }

    impl<T: ApexTimeP4> ApexTimeP4Ext for T {
        fn periodic_wait() -> Result<(), Error> {
            T::periodic_wait()?;
            Ok(())
        }

        fn get_time() -> SystemTime {
            T::get_time().into()
        }
    }

    /// Free extra functions for implementer of [ApexTimeP1]
    pub trait ApexTimeP1Ext: ApexTimeP1 + Sized {
        /// # Errors
        /// - [Error::InvalidMode]: calling process holds a mutex
        /// - [Error::InvalidMode]: calling process is error handler
        /// - [Error::InvalidParam]: `delay_time` is too large
        fn timed_wait(delay_time: Duration) -> Result<(), Error>;

        /// update deadline
        ///
        /// # Errors
        /// - [Error::InvalidParam]: `budget_time` is invalid
        /// - [Error::InvalidMode]: calling process is periodic AND calulated deadline exceeds next release point
        /// - [Error::NoAction]: calling process is error handler
        /// - [Error::NoAction]: our current operating mode is not [OperatingMode::Normal]
        fn replenish(budget_time: Duration) -> Result<(), Error>;
    }

    impl<T: ApexTimeP1> ApexTimeP1Ext for T {
        fn timed_wait(delay_time: Duration) -> Result<(), Error> {
            T::timed_wait(SystemTime::Normal(delay_time).into())?;
            Ok(())
        }

        fn replenish(budget_time: Duration) -> Result<(), Error> {
            T::replenish(SystemTime::Normal(budget_time).into())?;
            Ok(())
        }
    }
}
