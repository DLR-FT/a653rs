use crate::bindings::*;

pub trait ApexLimits {
    const SYSTEM_LIMIT_NUMBER_OF_PARTITIONS: ApexUnsigned = 32;
    const SYSTEM_LIMIT_NUMBER_OF_MESSAGES: MessageRange = 512;
    const SYSTEM_LIMIT_MESSAGE_SIZE: MessageSize = 8192;
    const SYSTEM_LIMIT_NUMBER_OF_PROCESSES: ApexUnsigned = 128;
    // const SYSTEM_LIMIT_NUMBER_OF_PROCESSES: ApexUnsigned = 2;
    const SYSTEM_LIMIT_NUMBER_OF_SAMPLING_PORTS: ApexUnsigned = 512;
    const SYSTEM_LIMIT_NUMBER_OF_QUEUING_PORTS: ApexUnsigned = 512;
    const SYSTEM_LIMIT_NUMBER_OF_BUFFERS: ApexUnsigned = 256;
    const SYSTEM_LIMIT_NUMBER_OF_BLACKBOARDS: ApexUnsigned = 256;
    const SYSTEM_LIMIT_NUMBER_OF_SEMAPHORES: ApexUnsigned = 256;
    const SYSTEM_LIMIT_NUMBER_OF_EVENTS: ApexUnsigned = 256;
    const SYSTEM_LIMIT_NUMBER_OF_MUTEXES: ApexUnsigned = 256;
}
