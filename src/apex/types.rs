pub mod basic {
    /// Max Length for Name Types
    ///
    /// According to ARINC653-P1, the maximum name length is always 32
    pub const MAX_NAME_LENGTH: usize = 32;
    /// C compatible function type
    pub type SystemAddress = extern "C" fn();
    /// Apex internal ReturnCode Type
    pub type ReturnCode = u32;
    /// Apex Name type using [MAX_NAME_LENGTH]
    pub type ApexName = [u8; MAX_NAME_LENGTH];

    // Base Types
    /// Apex Byte Type: 8-bit, 0..255
    pub type ApexByte = u8;
    /// Apex Integer Type: 32-bit, -2^31..2^31-1
    pub type ApexInteger = i32;
    /// Apex Unsigned Type: 32-bit, 0..4_294_967_295
    pub type ApexUnsigned = u32;
    /// Apex Long Integer Type: 64-bit: -2^63..2^63-1
    pub type ApexLongInteger = i64;

    /// Apex Message Size type: [ApexUnsigned]
    pub type MessageSize = ApexUnsigned;
    /// Apex Message Range type: [ApexUnsigned]
    pub type MessageRange = ApexUnsigned;

    /// APEX Error Return Code
    ///
    /// Basically the normal APEX Return Codes without the non-error variant
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

    /// Port Directions
    #[repr(u32)]
    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
    #[cfg_attr(feature = "strum", derive(strum::FromRepr))]
    pub enum PortDirection {
        /// Source/Sender Port
        Source = 0,
        /// Destination/Receiver Port
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

    /// Queuing Disciplines
    #[repr(u32)]
    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
    #[cfg_attr(feature = "strum", derive(strum::FromRepr))]
    pub enum QueuingDiscipline {
        /// First in/first out queue
        FIFO = 0,
        /// Priority queue
        Priority = 1,
    }

    impl TryFrom<ApexUnsigned> for QueuingDiscipline {
        type Error = ApexUnsigned;

        fn try_from(value: ApexUnsigned) -> Result<Self, Self::Error> {
            match value {
                0 => Ok(QueuingDiscipline::FIFO),
                1 => Ok(QueuingDiscipline::Priority),
                _ => Err(value),
            }
        }
    }

    /// Apex SystemTime Type: [ApexLongInteger]
    pub type ApexSystemTime = ApexLongInteger;
    /// [ApexSystemTime] value indicating infinite time
    pub const INFINITE_TIME_VALUE: ApexSystemTime = -1;

    /// ProcessorCore Id Type: [ApexInteger]
    pub type ProcessorCoreId = ApexInteger;
    /// [ProcessorCoreId] value indicating no preference
    pub const CORE_AFFINITY_NO_PREFERENCE: ProcessorCoreId = -1;
}

pub mod abstraction {
    use core::panic;
    use core::str::{FromStr, Utf8Error};
    use core::time::Duration;

    // Reexport important basic-types for downstream-user
    pub use super::basic::{
        ApexByte, ApexUnsigned, MessageRange, MessageSize, QueuingDiscipline, MAX_NAME_LENGTH,
    };
    use crate::bindings::*;

    /// Error Type used by abstracted functions.  
    /// Includes all Variants of [ErrorReturnCode] plus a [WriteError] and [ReadError] variant
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
        WriteError(WriteError),
        /// buffer is to small
        ReadError(ReadError),
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
        /// Infinite Time value
        Infinite,
        /// Normal positive Time value
        Normal(Duration),
    }

    impl SystemTime {
        /// Create new SystemTime from given [ApexSystemTime]
        pub fn new(time: ApexSystemTime) -> Self {
            time.into()
        }

        /// Returns Durations if this SystemTime is SystemTime::Normal
        ///
        /// Otherwise panics
        pub fn unwrap_duration(self) -> Duration {
            if let SystemTime::Normal(time) = self {
                return time;
            }
            panic!("Was Infinite")
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
        /// Converts [SystemTime] into [ApexSystemTime]  
        fn from(time: SystemTime) -> Self {
            if let SystemTime::Normal(time) = time {
                if let Ok(time) = ApexSystemTime::try_from(time.as_nanos()) {
                    return time;
                }
            }
            INFINITE_TIME_VALUE
        }
    }

    /// Convenient Abstraction Name Type  
    /// Uses [ApexName] internally
    #[derive(Clone, Debug, PartialEq, Eq)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
    pub struct Name(ApexName);

    impl Name {
        /// Create new [Name] from [ApexName]
        pub fn new(name: ApexName) -> Self {
            Name(name)
        }

        /// Get [str] from this
        pub fn to_str(&self) -> Result<&str, Utf8Error> {
            let nul_range_end = self
                .0
                .iter()
                .position(|&c| c == b'\0')
                .unwrap_or(self.0.len());
            core::str::from_utf8(&self.0[0..nul_range_end])
        }

        /// Dismantle this to its inner [ApexName] type
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
            let mut array_name = [0; MAX_NAME_LENGTH as usize];
            array_name[..s.len()].copy_from_slice(s.as_bytes());
            Ok(Self(array_name))
        }
    }

    /// Read Error indicating that the buffer was not guaranteed to fit a payload on a given read operation.  
    #[derive(Clone, Debug, PartialEq, Eq)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
    pub struct ReadError(usize);

    impl ReadError {
        /// Validate a buffer to fit at least a given size.  
        /// If not returns [Self] with the length of the passed buffer
        pub fn validate(
            size: MessageSize,
            buffer: &mut [ApexByte],
        ) -> Result<&mut [ApexByte], Self> {
            if usize::try_from(size)
                .map(|ss| buffer.len() < ss)
                .unwrap_or(true)
            {
                return Err(Self(buffer.len()));
            }
            Ok(buffer)
        }

        /// Returns the length of the buffer which was to small
        pub fn found_buffer_size(&self) -> usize {
            self.0
        }
    }

    impl From<ReadError> for Error {
        fn from(re: ReadError) -> Self {
            Error::ReadError(re)
        }
    }

    /// Write Error indicating that the buffer was to large for the given write operation.  
    #[derive(Clone, Debug, PartialEq, Eq)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
    pub struct WriteError(usize);

    impl WriteError {
        /// Validate a buffer to be at most as long as the given usize.  
        /// If not returns [Self] with the length of the passed buffer
        pub fn validate(size: MessageSize, buffer: &[ApexByte]) -> Result<&[ApexByte], Self> {
            if usize::try_from(size)
                .map(|ss| buffer.len() > ss)
                .unwrap_or(false)
                || buffer.is_empty()
            {
                return Err(Self(buffer.len()));
            }
            Ok(buffer)
        }

        /// Returns the length of the buffer which was to long
        pub fn found_buffer_size(&self) -> usize {
            self.0
        }
    }

    impl From<WriteError> for Error {
        fn from(we: WriteError) -> Self {
            Error::WriteError(we)
        }
    }
}
