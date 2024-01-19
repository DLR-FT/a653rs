use crate::apex::types::basic::*;

/// Hypervisor dependent limits
pub trait ApexLimits {
    /// ARINC653P1-5 APPENDIX D.1 maximum number of partitions for a configuration
    const SYSTEM_LIMIT_NUMBER_OF_PARTITIONS: ApexUnsigned = 32;
    /// ARINC653P1-5 2.3.5.7.6 maximum number of message in a port
    const SYSTEM_LIMIT_NUMBER_OF_MESSAGES: MessageRange = 512;
    /// ARINC653P1-5 2.3.5.7.5 maximum number of bytes in a single message
    const SYSTEM_LIMIT_MESSAGE_SIZE: MessageSize = 8192;
    /// ARINC653P1-5 3.3.2.3 maximum number of processes in partition
    ///
    /// Defaults:
    /// - ARINC653P1-5: 128
    /// - ARINC653P4: 2
    const SYSTEM_LIMIT_NUMBER_OF_PROCESSES: ApexUnsigned = 128;
    /// ARINC653P1-5 3.6.2.1.1 maximum number of sampling ports in partition
    const SYSTEM_LIMIT_NUMBER_OF_SAMPLING_PORTS: ApexUnsigned = 512;
    /// ARINC653P1-5 3.6.2.2.1 maximum number of queuing ports in partition
    const SYSTEM_LIMIT_NUMBER_OF_QUEUING_PORTS: ApexUnsigned = 512;
    /// ARINC653P1-5 3.7.2.1.1 maximum number of buffer in partition
    const SYSTEM_LIMIT_NUMBER_OF_BUFFERS: ApexUnsigned = 256;
    /// ARINC653P1-5 3.7.2.2.1 maximum number of blackboards in partition
    const SYSTEM_LIMIT_NUMBER_OF_BLACKBOARDS: ApexUnsigned = 256;
    /// ARINC653P1-5 3.7.2.3.1 maximum number of semaphores in partition
    const SYSTEM_LIMIT_NUMBER_OF_SEMAPHORES: ApexUnsigned = 256;
    /// ARINC653P1-5 3.7.2.4.1 maximum number of events in partition
    const SYSTEM_LIMIT_NUMBER_OF_EVENTS: ApexUnsigned = 256;
    /// ARINC653P1-5 3.7.2.5.1 maximum number of mutexes in partition
    const SYSTEM_LIMIT_NUMBER_OF_MUTEXES: ApexUnsigned = 256;
}
