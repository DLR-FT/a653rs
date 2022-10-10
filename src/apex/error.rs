pub mod basic {
    use crate::bindings::*;
    use crate::Locked;

    // TODO P4 extension

    // TODO attach to trait?
    pub const MAX_ERROR_MESSAGE_SIZE: usize = 128;

    pub type ErrorMessageSize = ApexInteger;
    pub type ErrorMessage = [ApexByte; MAX_ERROR_MESSAGE_SIZE];

    #[repr(u32)]
    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
    #[cfg_attr(feature = "strum", derive(strum::FromRepr))]
    pub enum ErrorCode {
        DeadlineMissed = 0,
        ApplicationError = 1,
        NumericError = 2,
        IllegalRequest = 3,
        StackOverflow = 4,
        MemoryViolation = 5,
        HardwareFault = 6,
        PowerFail = 7,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct ErrorStatus {
        pub failed_address: SystemAddress,
        pub failed_process_id: ProcessId,
        pub error_code: ErrorCode,
        pub length: ErrorMessageSize,
        pub message: ErrorMessage,
    }

    #[repr(u32)]
    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
    #[cfg_attr(feature = "strum", derive(strum::FromRepr))]
    pub enum ErrorHandlerConcurrencyControl {
        ProcessesPause = 0,
        ProcessesScheduled = 1,
    }

    pub trait ApexErrorP4 {
        #[cfg_attr(not(feature = "full_doc"), doc(hidden))]
        fn report_application_message<L: Locked>(
            message: &[ApexByte],
        ) -> Result<(), ErrorReturnCode>;

        #[cfg_attr(not(feature = "full_doc"), doc(hidden))]
        fn raise_application_error<L: Locked>(
            error_code: ErrorCode,
            message: &[ApexByte],
        ) -> Result<(), ErrorReturnCode>;
    }

    pub trait ApexErrorP1 {
        // Only during Warm/Cold-Start
        #[cfg_attr(not(feature = "full_doc"), doc(hidden))]
        fn create_error_handler<L: Locked>(
            entry_point: SystemAddress,
            stack_size: StackSize,
        ) -> Result<(), ErrorReturnCode>;

        #[cfg_attr(not(feature = "full_doc"), doc(hidden))]
        fn get_error_status<L: Locked>() -> Result<ErrorStatus, ErrorReturnCode>;

        // Only during Warm/Cold-Start
        #[cfg_attr(not(feature = "full_doc"), doc(hidden))]
        fn configure_error_handler<L: Locked>(
            concurrency_control: ErrorHandlerConcurrencyControl,
            processor_core_id: ProcessorCoreId,
        ) -> Result<(), ErrorReturnCode>;
    }
}
pub mod abstraction {
    // Reexport important basic-types for downstream-user
    pub use super::basic::{
        ApexErrorP1, ApexErrorP4, ErrorCode, ErrorHandlerConcurrencyControl, ErrorStatus,
        MAX_ERROR_MESSAGE_SIZE,
    };
    use crate::bindings::*;
    use crate::hidden::Key;
    use crate::prelude::*;

    pub trait ApexErrorP4Ext: ApexErrorP4 {
        fn report_application_message(message: &[ApexByte]) -> Result<(), Error>;

        fn raise_application_error(message: &[ApexByte]) -> Result<(), Error>;
    }

    pub trait ApexErrorP1Ext: ApexErrorP1 {
        fn error_status() -> Result<ErrorStatus, Error>;
    }

    impl<E: ApexErrorP4> ApexErrorP4Ext for E {
        fn report_application_message(message: &[ApexByte]) -> Result<(), Error> {
            E::report_application_message::<Key>(WriteError::validate(
                MAX_ERROR_MESSAGE_SIZE as u32,
                message,
            )?)?;
            Ok(())
        }

        fn raise_application_error(message: &[ApexByte]) -> Result<(), Error> {
            E::raise_application_error::<Key>(
                ErrorCode::ApplicationError,
                WriteError::validate(MAX_ERROR_MESSAGE_SIZE as u32, message)?,
            )?;
            Ok(())
        }
    }

    impl<E: ApexErrorP1> ApexErrorP1Ext for E {
        fn error_status() -> Result<ErrorStatus, Error> {
            Ok(E::get_error_status::<Key>()?)
        }
    }

    impl<E: ApexErrorP1> StartContext<E> {
        pub fn set_error_handler(
            &self,
            entry_point: SystemAddress,
            stack_size: StackSize,
        ) -> Result<(), Error> {
            E::create_error_handler::<Key>(entry_point, stack_size)?;
            Ok(())
        }

        pub fn configure_error_handler(
            &self,
            concurrency_control: ErrorHandlerConcurrencyControl,
            processor_core_id: ProcessorCoreId,
        ) -> Result<(), Error> {
            E::configure_error_handler::<Key>(concurrency_control, processor_core_id)?;
            Ok(())
        }
    }
}
