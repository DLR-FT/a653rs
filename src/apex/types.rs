/// ARINC653 types
pub mod basic {
    /// According to ARINC653-P1, the maximum name length is always 32
    pub const MAX_NAME_LENGTH: usize = 32;
    /// Apex internal ReturnCode Type
    pub type ReturnCode = u32;
    pub type ApexName = [u8; MAX_NAME_LENGTH];

    // Base Types
    pub type ApexByte = u8;
    pub type ApexInteger = i32;
    pub type ApexUnsigned = u32;
    pub type ApexLongInteger = i64;

    pub type MessageSize = ApexUnsigned;
    pub type MessageRange = ApexUnsigned;

    pub type WaitingRange = ApexInteger;

    /// The normal APEX Return Codes without the non-error variant
    #[repr(u32)]
    #[derive(Copy, Clone, Debug, PartialEq, Eq)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
    #[cfg_attr(feature = "strum", derive(strum::FromRepr))]
    pub enum ErrorReturnCode {
        /// status of system unaffected by request
        NoAction = 1,
        /// resource required by request unavailable
        NotAvailable = 2,
        /// invalid parameter specified in request
        InvalidParam = 3,
        /// parameter incompatible with configuration
        InvalidConfig = 4,
        /// request incompatible with current mode
        InvalidMode = 5,
        /// time-out tied up with request has expired
        TimedOut = 6,
    }

    impl ErrorReturnCode {
        /// Convenience function for gaining a Result from a given [ReturnCode]
        ///
        /// # Return Values for given [ReturnCode]
        ///
        /// - `0` => `Ok(())`
        /// - `1..=6` => `Err(Self)`
        /// - `7..` => `panic`
        pub fn from(from: ReturnCode) -> Result<(), Self> {
            use ErrorReturnCode::*;
            match from {
                0 => Ok(()),
                1 => Err(NoAction),
                2 => Err(NotAvailable),
                3 => Err(InvalidParam),
                4 => Err(InvalidConfig),
                5 => Err(InvalidMode),
                6 => Err(TimedOut),
                unexpected => panic!("{unexpected}"),
            }
        }
    }

    #[repr(u32)]
    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
    #[cfg_attr(feature = "strum", derive(strum::FromRepr))]
    pub enum PortDirection {
        Source = 0,
        Destination = 1,
    }

    impl TryFrom<ApexUnsigned> for PortDirection {
        type Error = ApexUnsigned;

        fn try_from(value: ApexUnsigned) -> Result<Self, Self::Error> {
            match value {
                0 => Ok(PortDirection::Source),
                1 => Ok(PortDirection::Destination),
                _ => Err(value),
            }
        }
    }

    #[repr(u32)]
    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
    #[cfg_attr(feature = "strum", derive(strum::FromRepr))]
    pub enum QueuingDiscipline {
        /// First in/first out queue
        Fifo = 0,
        /// Priority queue
        Priority = 1,
    }

    impl TryFrom<ApexUnsigned> for QueuingDiscipline {
        type Error = ApexUnsigned;

        fn try_from(value: ApexUnsigned) -> Result<Self, Self::Error> {
            match value {
                0 => Ok(QueuingDiscipline::Fifo),
                1 => Ok(QueuingDiscipline::Priority),
                _ => Err(value),
            }
        }
    }

    pub type ProcessorCoreId = ApexInteger;
    pub const CORE_AFFINITY_NO_PREFERENCE: ProcessorCoreId = -1;
}

pub mod abstraction {
    use core::str::{FromStr, Utf8Error};

    // Reexport important basic-types for downstream-user
    pub use super::basic::{
        ApexByte, ApexUnsigned, MessageRange, MessageSize, QueuingDiscipline, MAX_NAME_LENGTH,
    };
    use crate::bindings::*;

    /// Error Type used by abstracted functions.  
    /// Includes all Variants of [ErrorReturnCode] plus a `WriteError` and `ReadError` variant
    #[derive(Clone, Debug, PartialEq, Eq)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
    pub enum Error {
        /// status of system unaffected by request
        NoAction,
        /// resource required by request unavailable
        NotAvailable,
        /// invalid parameter specified in request
        InvalidParam,
        /// parameter incompatible with configuration
        InvalidConfig,
        /// request incompatible with current mode
        InvalidMode,
        /// time-out tied up with request has expired
        TimedOut,
        /// buffer got zero length or is to long
        WriteError,
        /// buffer is to small
        ReadError,
    }

    impl From<ErrorReturnCode> for Error {
        fn from(rc: ErrorReturnCode) -> Self {
            use Error::*;
            match rc {
                ErrorReturnCode::NoAction => NoAction,
                ErrorReturnCode::NotAvailable => NotAvailable,
                ErrorReturnCode::InvalidParam => InvalidParam,
                ErrorReturnCode::InvalidConfig => InvalidConfig,
                ErrorReturnCode::InvalidMode => InvalidMode,
                ErrorReturnCode::TimedOut => TimedOut,
            }
        }
    }

    /// Convenient Abstraction Name Type  
    /// Uses [ApexName] internally
    #[derive(Clone, Debug, PartialEq, Eq)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
    pub struct Name(ApexName);

    impl Name {
        pub const fn new(name: ApexName) -> Self {
            Name(name)
        }

        pub fn to_str(&self) -> Result<&str, Utf8Error> {
            let nul_range_end = self
                .0
                .iter()
                .position(|&c| c == b'\0')
                .unwrap_or(self.0.len());
            core::str::from_utf8(&self.0[0..nul_range_end])
        }

        pub fn into_inner(self) -> ApexName {
            self.0
        }
    }

    impl From<Name> for ApexName {
        fn from(val: Name) -> Self {
            val.0
        }
    }

    impl FromStr for Name {
        type Err = ApexUnsigned;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            if s.len() > MAX_NAME_LENGTH {
                return Err(s.len() as ApexUnsigned);
            }
            let mut array_name = [0; MAX_NAME_LENGTH];
            array_name[..s.len()].copy_from_slice(s.as_bytes());
            Ok(Self(array_name))
        }
    }

    pub trait BufferExt {
        fn validate_read(&mut self, size: MessageSize) -> Result<&mut Self, Error>;

        /// Validate a buffer to be at most as long as the given usize.  
        /// If not returns [Self] with the length of the passed buffer
        fn validate_write(&self, size: MessageSize) -> Result<&Self, Error>;
    }

    impl BufferExt for [ApexByte] {
        fn validate_read(&mut self, size: MessageSize) -> Result<&mut Self, Error> {
            if usize::try_from(size)
                .map(|ss| self.len() < ss)
                .unwrap_or(true)
            {
                return Err(Error::ReadError);
            }
            Ok(self)
        }

        fn validate_write(&self, size: MessageSize) -> Result<&Self, Error> {
            if usize::try_from(size)
                .map(|ss| self.len() > ss)
                .unwrap_or(false)
                || self.is_empty()
            {
                return Err(Error::WriteError);
            }
            Ok(self)
        }
    }
}
