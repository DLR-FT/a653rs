use a653rs::prelude::PartitionExt;
use a653rs_macros::partition;

mod deps;

fn main() {
    hello::Partition.run();
}

#[partition(crate::deps::dummy::DummyHypervisor)]
mod hello {
    #[sampling_out(name = "Ch1", msg_size = "10KB")]
    struct Channel1;

    #[sampling_in(refresh_period = "500ms")]
    #[sampling_in(msg_size = "25KB")]
    struct ChannelTwo;

    #[queuing_out(msg_count = 20, msg_size = "12KB", discipline = "FIFO")]
    struct Channel3;

    #[start(cold)]
    fn cold_start(ctx: start::Context) {
        warm_start(ctx);
    }

    #[start(warm)]
    fn warm_start(mut ctx: start::Context) {
        ctx.create_aperiodic2().unwrap().start().unwrap();
        ctx.create_periodic3().unwrap().start().unwrap();
        ctx.create_channel_1().unwrap();
        ctx.create_channel_two().unwrap();

        // Maybe we do not always want to initialize channel3
        // ctx.create_channel_3().unwrap();
    }

    #[aperiodic(
        name = "ap2",
        time_capacity = "Infinite",
        stack_size = "10KB",
        base_priority = 1,
        deadline = "Soft"
    )]
    fn aperiodic2(ctx: aperiodic2::Context) {
        ctx.get_time();
    }

    #[periodic(
        period = "10ms",
        time_capacity = "Infinite",
        stack_size = "10KB",
        base_priority = 1,
        deadline = "Hard"
    )]
    fn periodic3(_ctx: periodic3::Context) {}
}
