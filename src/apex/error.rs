/// bindings for ARINC653P1-5 3.8 health monitoring
pub mod basic {
    use crate::apex::process::basic::*;
    use crate::apex::types::basic::*;

    /// ARINC653P1-5 3.8.1 Maximum message size in bytes
    pub const MAX_ERROR_MESSAGE_SIZE: usize = 128;

    /// ARINC653P1-5 3.8.1
    pub type ErrorMessageSize = ApexInteger;
    /// ARINC653P1-5 3.8.1
    pub type ErrorMessage = [ApexByte; MAX_ERROR_MESSAGE_SIZE];

    /// ARINC653P1-5 3.8.1 Process level error
    #[repr(u32)]
    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
    #[cfg_attr(feature = "strum", derive(strum::FromRepr))]
    pub enum ErrorCode {
        DeadlineMissed = 0,
        /// error raised by [raise_application_error](crate::prelude::ApexErrorP4Ext::raise_application_error)
        ApplicationError = 1,
        NumericError = 2,
        /// unallowed syscall / OS request
        IllegalRequest = 3,
        StackOverflow = 4,
        MemoryViolation = 5,
        /// I/O error
        HardwareFault = 6,
        /// info for saving data before power off
        PowerFail = 7,
    }

    /// ARINC653P1-5 3.8.1
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct ErrorStatus {
        /// related implementation dependent address
        pub failed_address: SystemAddress,
        pub failed_process_id: ProcessId,
        pub error_code: ErrorCode,
        /// error message length
        pub length: ErrorMessageSize,
        pub message: ErrorMessage,
    }

    /// ARINC653P1-5 3.8.1
    #[repr(u32)]
    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
    #[cfg_attr(feature = "strum", derive(strum::FromRepr))]
    pub enum ErrorHandlerConcurrencyControl {
        /// processes are paused when error handler is active
        ProcessesPause = 0,
        /// processes run parallel to error handler
        ProcessesScheduled = 1,
    }

    /// ARINC653P4 3.8.2 required functions for health monitoring functionality
    pub trait ApexErrorP4 {
        /// APEX653P4 3.8.2.1 report message to health monitor
        ///
        /// # Errors
        /// - [ErrorReturnCode::InvalidParam]: `message` is too large
        #[cfg_attr(not(feature = "full_doc"), doc(hidden))]
        fn report_application_message(message: &[ApexByte]) -> Result<(), ErrorReturnCode>;

        /// APEX653P4 3.8.2.4 trigger error handler process
        ///
        /// # Errors
        /// - [ErrorReturnCode::InvalidParam]: `message` is larger than [MAX_ERROR_MESSAGE_SIZE]
        /// - [ErrorReturnCode::InvalidParam]: `error_code` is not [ErrorCode::ApplicationError]
        #[cfg_attr(not(feature = "full_doc"), doc(hidden))]
        fn raise_application_error(
            error_code: ErrorCode,
            message: &[ApexByte],
        ) -> Result<(), ErrorReturnCode>;
    }

    /// ARINC653P1-5 3.8.2 required functions for health monitoring functionality
    pub trait ApexErrorP1 {
        /// APEX653P1-5 3.8.2.2
        ///
        /// # Errors
        /// - [ErrorReturnCode::NoAction]: error handler exists already
        /// - [ErrorReturnCode::InvalidConfig]: not enough memory is available
        /// - [ErrorReturnCode::InvalidConfig]: `stack_size` is too large
        /// - [ErrorReturnCode::InvalidMode]: our current operating mode is [OperatingMode::Normal](crate::prelude::OperatingMode::Normal)
        #[cfg_attr(not(feature = "full_doc"), doc(hidden))]
        fn create_error_handler(
            entry_point: SystemAddress,
            stack_size: StackSize,
        ) -> Result<(), ErrorReturnCode>;

        /// APEX653P1-5 3.8.2.3 get current error status
        ///
        /// # Errors
        /// - [ErrorReturnCode::InvalidConfig]: the calling process is not an error handler
        /// - [ErrorReturnCode::NoAction]: no error exists right now
        #[cfg_attr(not(feature = "full_doc"), doc(hidden))]
        fn get_error_status() -> Result<ErrorStatus, ErrorReturnCode>;

        /// APEX653P1-5 3.8.2.5
        ///
        /// # Errors
        /// - [ErrorReturnCode::InvalidConfig]: no error handler exists
        /// - [ErrorReturnCode::InvalidMode]: our current operating mode is [OperatingMode::Normal](crate::prelude::OperatingMode::Normal)
        #[cfg_attr(not(feature = "full_doc"), doc(hidden))]
        fn configure_error_handler(
            concurrency_control: ErrorHandlerConcurrencyControl,
            processor_core_id: ProcessorCoreId,
        ) -> Result<(), ErrorReturnCode>;
    }
}

/// abstraction for ARINC653P1-5 3.8 health monitoring
pub mod abstraction {
    use super::basic::{ApexErrorP1, ApexErrorP4};
    // Reexport important basic-types for downstream-user
    pub use super::basic::{
        ErrorCode, ErrorHandlerConcurrencyControl, ErrorStatus, MAX_ERROR_MESSAGE_SIZE,
    };
    use crate::prelude::*;

    /// Free extra functions for implementer of [ApexErrorP4]
    pub trait ApexErrorP4Ext: ApexErrorP4 {
        /// report message to health monitor
        ///
        /// # Errors
        /// - [Error::InvalidParam]: `message` is too large
        fn report_application_message(message: &[ApexByte]) -> Result<(), Error>;

        /// trigger error handler process
        ///
        /// # Errors
        /// - [Error::InvalidParam]: `message` is larger than [MAX_ERROR_MESSAGE_SIZE]
        fn raise_application_error(message: &[ApexByte]) -> Result<(), Error>;
    }

    /// Free extra functions for implementer of [ApexErrorP1]
    pub trait ApexErrorP1Ext: ApexErrorP1 {
        /// get current error status
        ///
        /// # Errors
        /// - [Error::InvalidConfig]: the calling process is not an error handler
        /// - [Error::NoAction]: no error exists right now
        fn error_status() -> Result<ErrorStatus, Error>;
    }

    impl<E: ApexErrorP4> ApexErrorP4Ext for E {
        fn report_application_message(message: &[ApexByte]) -> Result<(), Error> {
            E::report_application_message(message.validate_write(MAX_ERROR_MESSAGE_SIZE as u32)?)?;
            Ok(())
        }

        fn raise_application_error(message: &[ApexByte]) -> Result<(), Error> {
            E::raise_application_error(
                ErrorCode::ApplicationError,
                message.validate_write(MAX_ERROR_MESSAGE_SIZE as u32)?,
            )?;
            Ok(())
        }
    }

    impl<E: ApexErrorP1> ApexErrorP1Ext for E {
        fn error_status() -> Result<ErrorStatus, Error> {
            Ok(E::get_error_status()?)
        }
    }

    impl<E: ApexErrorP1> StartContext<E> {
        /// # Errors
        /// - [Error::NoAction]: error handler exists already
        /// - [Error::InvalidConfig]: not enough memory is available
        /// - [Error::InvalidConfig]: `stack_size` is too large
        pub fn set_error_handler(
            &self,
            entry_point: SystemAddress,
            stack_size: StackSize,
        ) -> Result<(), Error> {
            E::create_error_handler(entry_point, stack_size)?;
            Ok(())
        }

        /// # Errors
        /// - [Error::InvalidConfig]: no error handler exists
        pub fn configure_error_handler(
            &self,
            concurrency_control: ErrorHandlerConcurrencyControl,
            processor_core_id: ProcessorCoreId,
        ) -> Result<(), Error> {
            E::configure_error_handler(concurrency_control, processor_core_id)?;
            Ok(())
        }
    }
}
