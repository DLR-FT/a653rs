use a653rs::bindings::*;

pub struct Dummy;

impl ApexPartitionP4 for Dummy {
    fn get_partition_status<L: Locked>() -> ApexPartitionStatus {
        todo!()
    }

    fn set_partition_mode<L: Locked>(
        _operating_mode: OperatingMode,
    ) -> Result<(), ErrorReturnCode> {
        todo!()
    }
}

impl ApexQueuingPortP1 for Dummy {
    fn get_queuing_port_id<L: Locked>(
        _queuing_port_name: QueuingPortName,
    ) -> Result<QueuingPortId, ErrorReturnCode> {
        todo!()
    }
}

impl ApexQueuingPortP4 for Dummy {
    fn create_queuing_port<L: Locked>(
        _queuing_port_name: QueuingPortName,
        _max_message_size: MessageSize,
        _max_nb_message: MessageRange,
        _port_direction: PortDirection,
        _queuing_discipline: QueuingDiscipline,
    ) -> Result<QueuingPortId, ErrorReturnCode> {
        todo!()
    }

    fn send_queuing_message<L: Locked>(
        _queuing_port_id: QueuingPortId,
        _message: &[ApexByte],
        _time_out: ApexSystemTime,
    ) -> Result<(), ErrorReturnCode> {
        todo!()
    }

    unsafe fn receive_queuing_message<L: Locked>(
        _queuing_port_id: QueuingPortId,
        _time_out: ApexSystemTime,
        _message: &mut [ApexByte],
    ) -> Result<MessageSize, ErrorReturnCode> {
        todo!()
    }

    fn get_queuing_port_status<L: Locked>(
        _queuing_port_id: QueuingPortId,
    ) -> Result<QueuingPortStatus, ErrorReturnCode> {
        todo!()
    }

    fn clear_queuing_port<L: Locked>(
        _queuing_port_id: QueuingPortId,
    ) -> Result<(), ErrorReturnCode> {
        todo!()
    }
}

impl ApexSamplingPortP4 for Dummy {
    fn create_sampling_port<L: Locked>(
        _sampling_port_name: SamplingPortName,
        _max_message_size: MessageSize,
        _port_direction: PortDirection,
        _refresh_period: ApexSystemTime,
    ) -> Result<SamplingPortId, ErrorReturnCode> {
        todo!()
    }

    fn write_sampling_message<L: Locked>(
        _sampling_port_id: SamplingPortId,
        _message: &[ApexByte],
    ) -> Result<(), ErrorReturnCode> {
        todo!()
    }

    unsafe fn read_sampling_message<L: Locked>(
        _sampling_port_id: SamplingPortId,
        _message: &mut [ApexByte],
    ) -> Result<(Validity, MessageSize), ErrorReturnCode> {
        todo!()
    }
}

impl ApexSamplingPortP1 for Dummy {
    fn get_sampling_port_id<L: Locked>(
        _sampling_port_name: SamplingPortName,
    ) -> Result<SamplingPortId, ErrorReturnCode> {
        todo!()
    }

    fn get_sampling_port_status<L: Locked>(
        _sampling_port_id: SamplingPortId,
    ) -> Result<ApexSamplingPortStatus, ErrorReturnCode> {
        todo!()
    }
}

impl ApexProcessP4 for Dummy {
    fn create_process<L: Locked>(
        _attributes: &ApexProcessAttribute,
    ) -> Result<ProcessId, ErrorReturnCode> {
        todo!()
    }

    fn start<L: Locked>(_process_id: ProcessId) -> Result<(), ErrorReturnCode> {
        todo!()
    }
}

impl ApexProcessP1 for Dummy {
    fn set_priority<L: Locked>(
        _process_id: ProcessId,
        _priority: Priority,
    ) -> Result<(), ErrorReturnCode> {
        todo!()
    }

    fn suspend_self<L: Locked>(_time_out: ApexSystemTime) -> Result<(), ErrorReturnCode> {
        todo!()
    }

    fn suspend<L: Locked>(_process_id: ProcessId) -> Result<(), ErrorReturnCode> {
        todo!()
    }

    fn resume<L: Locked>(_process_id: ProcessId) -> Result<(), ErrorReturnCode> {
        todo!()
    }

    fn stop_self<L: Locked>() {
        todo!()
    }

    fn stop<L: Locked>(_process_id: ProcessId) -> Result<(), ErrorReturnCode> {
        todo!()
    }

    fn delayed_start<L: Locked>(
        _process_id: ProcessId,
        _delay_time: ApexSystemTime,
    ) -> Result<(), ErrorReturnCode> {
        todo!()
    }

    fn lock_preemption<L: Locked>() -> Result<LockLevel, ErrorReturnCode> {
        todo!()
    }

    fn unlock_preemption<L: Locked>() -> Result<LockLevel, ErrorReturnCode> {
        todo!()
    }

    fn get_my_id<L: Locked>() -> Result<ProcessId, ErrorReturnCode> {
        todo!()
    }

    fn get_process_id<L: Locked>(_process_name: ProcessName) -> Result<ProcessId, ErrorReturnCode> {
        todo!()
    }

    fn get_process_status<L: Locked>(
        _process_id: ProcessId,
    ) -> Result<ApexProcessStatus, ErrorReturnCode> {
        todo!()
    }

    fn initialize_process_core_affinity<L: Locked>(
        _process_id: ProcessId,
        _processor_core_id: ProcessorCoreId,
    ) -> Result<(), ErrorReturnCode> {
        todo!()
    }

    fn get_my_processor_core_id<L: Locked>() -> ProcessorCoreId {
        todo!()
    }

    fn get_my_index<L: Locked>() -> Result<ProcessIndex, ErrorReturnCode> {
        todo!()
    }
}

impl ApexTimeP4 for Dummy {
    fn periodic_wait() -> Result<(), ErrorReturnCode> {
        todo!()
    }

    fn get_time() -> ApexSystemTime {
        todo!()
    }
}
