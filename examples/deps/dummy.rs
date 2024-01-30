use a653rs::bindings::*;

pub struct DummyHypervisor;

impl ApexPartitionP4 for DummyHypervisor {
    fn get_partition_status() -> ApexPartitionStatus {
        todo!()
    }

    fn set_partition_mode(_operating_mode: OperatingMode) -> Result<(), ErrorReturnCode> {
        todo!()
    }
}

impl ApexQueuingPortP1 for DummyHypervisor {
    fn get_queuing_port_id(
        _queuing_port_name: QueuingPortName,
    ) -> Result<QueuingPortId, ErrorReturnCode> {
        todo!()
    }
}

impl ApexQueuingPortP4 for DummyHypervisor {
    fn create_queuing_port(
        _queuing_port_name: QueuingPortName,
        _max_message_size: MessageSize,
        _max_nb_message: MessageRange,
        _port_direction: PortDirection,
        _queuing_discipline: QueuingDiscipline,
    ) -> Result<QueuingPortId, ErrorReturnCode> {
        todo!()
    }

    fn send_queuing_message(
        _queuing_port_id: QueuingPortId,
        _message: &[ApexByte],
        _time_out: ApexSystemTime,
    ) -> Result<(), ErrorReturnCode> {
        todo!()
    }

    unsafe fn receive_queuing_message(
        _queuing_port_id: QueuingPortId,
        _time_out: ApexSystemTime,
        _message: &mut [ApexByte],
    ) -> Result<(MessageSize, QueueOverflow), ErrorReturnCode> {
        todo!()
    }

    fn get_queuing_port_status(
        _queuing_port_id: QueuingPortId,
    ) -> Result<QueuingPortStatus, ErrorReturnCode> {
        todo!()
    }

    fn clear_queuing_port(_queuing_port_id: QueuingPortId) -> Result<(), ErrorReturnCode> {
        todo!()
    }
}

impl ApexSamplingPortP4 for DummyHypervisor {
    fn create_sampling_port(
        _sampling_port_name: SamplingPortName,
        _max_message_size: MessageSize,
        _port_direction: PortDirection,
        _refresh_period: ApexSystemTime,
    ) -> Result<SamplingPortId, ErrorReturnCode> {
        todo!()
    }

    fn write_sampling_message(
        _sampling_port_id: SamplingPortId,
        _message: &[ApexByte],
    ) -> Result<(), ErrorReturnCode> {
        todo!()
    }

    unsafe fn read_sampling_message(
        _sampling_port_id: SamplingPortId,
        _message: &mut [ApexByte],
    ) -> Result<(Validity, MessageSize), ErrorReturnCode> {
        todo!()
    }
}

impl ApexSamplingPortP1 for DummyHypervisor {
    fn get_sampling_port_id(
        _sampling_port_name: SamplingPortName,
    ) -> Result<SamplingPortId, ErrorReturnCode> {
        todo!()
    }

    fn get_sampling_port_status(
        _sampling_port_id: SamplingPortId,
    ) -> Result<ApexSamplingPortStatus, ErrorReturnCode> {
        todo!()
    }
}

impl ApexProcessP4 for DummyHypervisor {
    fn create_process(_attributes: &ApexProcessAttribute) -> Result<ProcessId, ErrorReturnCode> {
        todo!()
    }

    fn start(_process_id: ProcessId) -> Result<(), ErrorReturnCode> {
        todo!()
    }
}

impl ApexProcessP1 for DummyHypervisor {
    fn set_priority(_process_id: ProcessId, _priority: Priority) -> Result<(), ErrorReturnCode> {
        todo!()
    }

    fn suspend_self(_time_out: ApexSystemTime) -> Result<(), ErrorReturnCode> {
        todo!()
    }

    fn suspend(_process_id: ProcessId) -> Result<(), ErrorReturnCode> {
        todo!()
    }

    fn resume(_process_id: ProcessId) -> Result<(), ErrorReturnCode> {
        todo!()
    }

    fn stop_self() {
        todo!()
    }

    fn stop(_process_id: ProcessId) -> Result<(), ErrorReturnCode> {
        todo!()
    }

    fn delayed_start(
        _process_id: ProcessId,
        _delay_time: ApexSystemTime,
    ) -> Result<(), ErrorReturnCode> {
        todo!()
    }

    fn lock_preemption() -> Result<LockLevel, ErrorReturnCode> {
        todo!()
    }

    fn unlock_preemption() -> Result<LockLevel, ErrorReturnCode> {
        todo!()
    }

    fn get_my_id() -> Result<ProcessId, ErrorReturnCode> {
        todo!()
    }

    fn get_process_id(_process_name: ProcessName) -> Result<ProcessId, ErrorReturnCode> {
        todo!()
    }

    fn get_process_status(_process_id: ProcessId) -> Result<ApexProcessStatus, ErrorReturnCode> {
        todo!()
    }

    fn initialize_process_core_affinity(
        _process_id: ProcessId,
        _processor_core_id: ProcessorCoreId,
    ) -> Result<(), ErrorReturnCode> {
        todo!()
    }

    fn get_my_processor_core_id() -> ProcessorCoreId {
        todo!()
    }

    fn get_my_index() -> Result<ProcessIndex, ErrorReturnCode> {
        todo!()
    }
}

impl ApexTimeP4 for DummyHypervisor {
    fn periodic_wait() -> Result<(), ErrorReturnCode> {
        todo!()
    }

    fn get_time() -> ApexSystemTime {
        todo!()
    }
}

impl ApexErrorP4 for DummyHypervisor {
    fn report_application_message(_message: &[ApexByte]) -> Result<(), ErrorReturnCode> {
        todo!()
    }

    fn raise_application_error(
        _error_code: ErrorCode,
        _message: &[ApexByte],
    ) -> Result<(), ErrorReturnCode> {
        todo!()
    }
}
