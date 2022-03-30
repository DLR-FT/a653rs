pub mod basic {
    use crate::bindings::*;

    pub trait ApexTimeP4 {
        fn periodic_wait() -> Result<(), ErrorReturnCode>;
        fn get_time() -> ApexSystemTime;
    }

    pub trait ApexTimeP1: ApexTimeP4 {
        fn timed_wait<L: Locked>(delay_time: ApexSystemTime);
        fn replenish<L: Locked>(budget_time: ApexSystemTime) -> Result<(), ErrorReturnCode>;
    }
}

pub mod abstraction {
    use core::time::Duration;

    use super::basic::{ApexTimeP1, ApexTimeP4};
    use crate::hidden::Key;
    use crate::prelude::*;

    pub trait ApexTimeP4Ext: ApexTimeP4 + Sized {
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

    pub trait ApexTimeP1Ext: ApexTimeP1 + Sized {
        fn timed_wait(delay_time: Duration);

        fn replenish(budget_time: Duration) -> Result<(), Error>;
    }

    impl<T: ApexTimeP1> ApexTimeP1Ext for T {
        fn timed_wait(delay_time: Duration) {
            T::timed_wait::<Key>(SystemTime::Normal(delay_time).into())
        }

        fn replenish(budget_time: Duration) -> Result<(), Error> {
            T::replenish::<Key>(SystemTime::Normal(budget_time).into())?;
            Ok(())
        }
    }
}
