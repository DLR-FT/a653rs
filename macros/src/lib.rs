#![deny(rustdoc::broken_intra_doc_links)]
use partition::Partition;
use proc_macro::TokenStream;
use syn::{parse_macro_input, ItemMod, TypePath};

mod generate;
mod parse;
mod partition;

/// Convenience macro for simpler partition development with less pitfalls
///
/// For using this macro a module is annotated with the [`partition()`] attribute.
/// Inside of this module, start functions, processes, as well as channels can be defined using attributes.
///
/// [`partition()`]: macro@partition#attribute-partition
///
/// # Example
/// ```no_run
/// use a653rs::prelude::PartitionExt;
/// use a653rs_macros::partition;
///
/// # // TODO include example/partition.rs
/// # #[path = "../../examples/deps/dummy.rs"]
/// mod dummy;
///
/// fn main() {
///     example::Partition.run();
/// }
///
/// #[partition(crate::dummy::DummyHypervisor)]
/// mod example {
///     #[sampling_out(name = "Ch1", msg_size = "10KB")]
///     struct Channel1;
///
///     #[sampling_in(refresh_period = "500ms")]
///     #[sampling_in(msg_size = "25KB")]
///     struct ChannelTwo;
///
///     #[queuing_out(msg_count = 20, msg_size = "12KB", discipline = "FIFO")]
///     struct Channel3;
///
///     #[start(cold)]
///     fn cold_start(ctx: start::Context) {
///         warm_start(ctx);
///     }
///
///     #[start(warm)]
///     fn warm_start(mut ctx: start::Context) {
///         ctx.create_aperiodic2().unwrap().start().unwrap();
///         ctx.create_periodic3().unwrap().start().unwrap();
///         ctx.create_channel_1().unwrap();
///         ctx.create_channel_two().unwrap();
///
///         // Maybe we do not always want to initialize channel3
///         // ctx.create_channel_3().unwrap();
///     }
///
///     #[aperiodic(
///         name = "ap2",
///         time_capacity = "2ms",
///         stack_size = "10KB",
///         base_priority = 1,
///         deadline = "Soft"
///     )]
///     fn aperiodic2(ctx: aperiodic2::Context) {
///         ctx.get_time();
///     }
///
///     #[periodic(
///         period = "10ms",
///         time_capacity = "5ms",
///         stack_size = "10KB",
///         base_priority = 1,
///         deadline = "Hard"
///     )]
///     fn periodic3(ctx: periodic3::Context) {}
/// }
/// ```
///
/// # Attribute `#[partition()]`
///
/// The [`partition()`] attribute marks the entry point of this macro.
/// It is meant to be used on a module containing the partition.
///
/// When the attribute is used correctly, inside of the module a `Partition` struct is made available.
/// This `Partition` struct can then be used in i.e the `main` function for running the partition.
///
/// ## Requirements
///
/// #### #[partition(HYPERVISOR)]
///
/// - *HYPERVISOR*: the full path to the used hypervisor
///
/// #### Module
/// - [`start(cold)`] and [`start(cold)`]
/// - For calling `run()` on the `Partition` struct, HYPERVISOR must implement `a653rs::prelude::PartitionExt`
///
/// [`start(cold)`]: macro@partition#attributes-startcold-and-startwarm
/// [`start(warm)`]: macro@partition#attributes-startcold-and-startwarm
///
/// ## Flexibility
///
/// - The module name can be anything
///
/// ## Example
/// ```no_run
/// use a653rs::prelude::PartitionExt;
/// use a653rs_macros::partition;
/// # #[path = "../../examples/deps/dummy.rs"]
/// # mod dummy;
///
/// fn main() {
///     example::Partition.run();
/// }
///
/// #[partition(crate::dummy::DummyHypervisor)]
/// mod example {
///     #[start(cold)]
///     fn cold_start(ctx: start::Context) { }
///
///     #[start(warm)]
///     fn warm_start(ctx: start::Context) { }
/// }
/// ```
///
/// ## Attributes `start(cold)` and `start(warm)`
///
/// [`start(cold)`] and [`start(warm)`] are used for the start functions of the partition.
/// Inside these functions, the `start::Context` provides simple functions
/// for initializing processes, channels and using apex functionalities of the provided hypervisor.
///
/// ## Requirements
///
/// - the start functions must require solely the `Context` parameter
///
/// ## Flexibility
///
/// - The identifier of the functions can be anything
/// - The identifier of the `start::Context` can be anything
///
/// ## Example
/// ```no_run
/// # use a653rs::prelude::PartitionExt;
/// # use a653rs_macros::partition;
/// # #[path = "../../examples/deps/dummy.rs"]
/// # mod dummy;
/// # fn main() {
/// #     example::Partition.run();
/// # }
/// # #[partition(crate::dummy::DummyHypervisor)]
/// # mod example {
/// #[start(cold)]
/// fn cold_start(ctx: start::Context) {
///     let status = ctx.get_partition_status();
/// }
///
/// #[start(warm)]
/// fn warm_start(ctx: start::Context) {
///     cold_start(ctx);
/// }
/// # }
/// ```
///
/// # Attributes `periodic()` and `aperiodic()`
///
/// Two types of processes are available: periodic and aperiodic processes.  
///
/// Functions with either the [`periodic()`] or [`aperiodic()`] attribute use a `Context` parameter for interacting with the rest of the partition.
/// This `Context` contains fields for all defined channels and processes as well as functions provided by the used hypervisor.
///
/// When a process is defined, a `create_NAME()` function is made available on the `start::Context` struct in [`start(cold)`] and [`start(warm)`].
/// This function must be called in order to initialize the process.
/// Also, these create functions return a reference to the process on success.
/// For the process to be scheduled, the `start()` function must be called on this reference.
///
/// [`periodic()`]: macro@partition#attributes-periodic-and-aperiodic
/// [`aperiodic()`]: macro@partition#attributes-periodic-and-aperiodic
///
/// ## Requirements
///
/// - the functions must require solely the `Context` parameter
///   - the module path of the `Context` is the name of the function
///
/// #### #[periodic(NAME, PERIOD, TIME_CAPACITY, STACK_SIZE, BASE_PRIORITY, DEADLINE)]
///
/// - **NAME**: name used for internal apex calls (optional)
/// - **PERIOD**: time like ["10ms", "16s", "18m", ...](https://crates.io/crates/humantime)
///   - Suggested value for P4: equal to the partition period
/// - **TIME_CAPACITY**: either "Infinite" or a time like ["10ms", "16s", "18m", ...](https://crates.io/crates/humantime)
///   - Suggested value for P4: equal to the partition duration
/// - **STACK_SIZE**: size like ["10KB", "16kiB", "12Mb", ...](https://crates.io/crates/bytesize)
/// - **BASE_PRIORITY**: [i32]
///   - Suggested value for P4: lower than the base priority of the aperiodic process
/// - **DEADLINE**: either "Hard" or "Soft"
///
/// #### #[aperiodic(NAME, TIME_CAPACITY, STACK_SIZE, BASE_PRIORITY, DEADLINE)]
///
/// - **NAME**: name used for internal apex calls (optional)  
/// - **TIME_CAPACITY**: either "Infinite" or a time like ["10ms", "16s", "18m", ...](https://crates.io/crates/humantime)
///   - Suggested value for P4: equal to the partition duration
/// - **STACK_SIZE**: size like ["10KB", "16kiB", "12Mb", ...](https://crates.io/crates/bytesize)
/// - **BASE_PRIORITY**: [i32]
///   - Suggested value for P4: higher than the base priority of the periodic process
/// - **DEADLINE**: either "Hard" or "Soft"
///
/// ## Flexibility
///
/// - The identifier of the functions can be anything
/// - The identifier of the `Context` can be anything
///
/// ## Example
/// ```no_run
/// # use a653rs::prelude::PartitionExt;
/// # use a653rs_macros::partition;
/// # #[path = "../../examples/deps/dummy.rs"]
/// # mod dummy;
/// # fn main() {
/// #     example::Partition.run();
/// # }
/// # #[partition(crate::dummy::DummyHypervisor)]
/// # mod example {
/// #[start(cold)]
/// fn cold_start(ctx: start::Context) {
///     warm_start(ctx);
/// }
///
/// #[start(warm)]
/// fn warm_start(mut ctx: start::Context) {
///     ctx.create_aperiodic2().unwrap().start().unwrap();
///     ctx.create_periodic3().unwrap().start().unwrap();
/// }
///
/// #[aperiodic(
///     name = "ap2",
///     time_capacity = "Infinite",
///     stack_size = "10KB",
///     base_priority = 1,
///     deadline = "Soft"
/// )]
/// fn aperiodic2(ctx: aperiodic2::Context) {
///     ctx.get_time();
///     ctx.periodic3.unwrap().stop();
/// }
///
/// #[periodic(
///     period = "10ms",
///     time_capacity = "Infinite",
///     stack_size = "10KB",
///     base_priority = 1,
///     deadline = "Hard"
/// )]
/// fn periodic3(ctx: periodic3::Context) {
///     let status = ctx.proc_self.status();
///     ctx.report_application_message(b"Hello World").unwrap()
/// }
/// # }
/// ```
///
/// # Attributes `sampling_out()`, `sampling_in()`, `queuing_out()` and `queuing_in()`
///
/// Two types of channel are available: sampling and queuing ports.  
///
/// Structs with [`sampling_out()`], [`sampling_in()`], [`queuing_out()`] and [`queuing_in()`] attribute define channel.
///
/// When a channel is defined, a `create_NAME()` function is made available on the `start::Context` struct in [`start(cold)`] and [`start(warm)`].
/// This function must be called in order to initialize the channel.
/// Also a field for each created channel is made available on the `Context` of each [`periodic()`] and [`aperiodic()`] process.
///
/// [`sampling_out()`]: macro@partition#attributes-sampling_out-sampling_in-queuing_out-and-queuing_in
/// [`sampling_in()`]: macro@partition#attributes-sampling_out-sampling_in-queuing_out-and-queuing_in
/// [`queuing_out()`]: macro@partition#attributes-sampling_out-sampling_in-queuing_out-and-queuing_in
/// [`queuing_in()`]: macro@partition#attributes-sampling_out-sampling_in-queuing_out-and-queuing_in
///
/// ## Requirements
///
/// #### #[sampling_out(NAME, MSG_SIZE)]
///
/// - **NAME**: name used for internal apex calls (optional)
/// - **MSG_SIZE**: size like ["10KB", "16kiB", "12Mb", ...](https://crates.io/crates/bytesize)
///
/// #### #[sampling_in(NAME, MSG_SIZE, REFRESH_PERIOD)]
///
/// - **NAME**: name used for internal apex calls (optional)  
/// - **MSG_SIZE**: size like ["10KB", "16kiB", "12Mb", ...](https://crates.io/crates/bytesize)
/// - **REFRESH_PERIOD**: time like ["10ms", "16s", "18m", ...](https://crates.io/crates/humantime)
///
/// #### #[queuing_out(NAME, MSG_COUNT, MSG_SIZE, DISCIPLINE)]
///
/// - **NAME**: name used for internal apex calls (optional)
/// - **MSG_COUNT**: [u32]
/// - **MSG_SIZE**: size like ["10KB", "16kiB", "12Mb", ...](https://crates.io/crates/bytesize)
/// - **DISCIPLINE**: either "FIFO" or "Priority"
///
/// #### #[queuing_in(NAME, MSG_COUNT, MSG_SIZE, DISCIPLINE)]
///
/// - **NAME**: name used for internal apex calls (optional)  
/// - **MSG_COUNT**: [u32]
/// - **MSG_SIZE**: size like ["10KB", "16kiB", "12Mb", ...](https://crates.io/crates/bytesize)
/// - **REFRESH_PERIOD**: time like ["10ms", "16s", "18m", ...](https://crates.io/crates/humantime)
/// - **DISCIPLINE**: either "FIFO" or "Priority"
///
/// ## Flexibility
///
/// - The identifier of the struct can be anything
///
/// ## Example
/// ```no_run
/// # use a653rs::prelude::PartitionExt;
/// # use a653rs_macros::partition;
/// # #[path = "../../examples/deps/dummy.rs"]
/// # mod dummy;
/// # fn main() {
/// #     example::Partition.run();
/// # }
/// # #[partition(crate::dummy::DummyHypervisor)]
/// # mod example {
/// #[sampling_out(name = "Ch1", msg_size = "10KB")]
/// struct Channel1;
///
/// #[sampling_in(refresh_period = "500ms")]
/// #[sampling_in(msg_size = "25KB")]
/// struct ChannelTwo;
///
/// #[queuing_out(msg_count = 20, msg_size = "12KB", discipline = "FIFO")]
/// struct Channel3;
///
/// #[queuing_in(name = "ch_3", msg_count = 20, msg_size = "12KB", discipline = "Priority")]
/// struct LastChannel;
///
/// #[start(cold)]
/// fn cold_start(ctx: start::Context) {
///     warm_start(ctx);
/// }
///
/// #[start(warm)]
/// fn warm_start(mut ctx: start::Context) {
///     ctx.create_channel_1().unwrap();
///     ctx.create_channel_two().unwrap();
///     ctx.create_channel_3().unwrap();
///     ctx.create_last_channel().unwrap();
/// }
/// # }
/// ```
///
///
///
#[proc_macro_attribute]
pub fn partition(args: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemMod);
    // Right now we only expect the Identifier of the used Hypervisor here
    let args = parse_macro_input!(args as TypePath);

    // TODO allow only for a single partition per project

    Partition::expand_partition(input, args)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
