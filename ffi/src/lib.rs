use apex::partition::generate_partition_ffi;
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use syn::{parse_macro_input, Path};

mod apex;

/// Generator of C-FFI functions
///
/// # Example
/// ```no_run
/// # #[path = "../../examples/deps/dummy.rs"]
/// mod dummy;
///
/// a653rs_ffi::ffi!(crate::dummy::DymmyHypervisor);
///
/// ```

#[proc_macro]
pub fn ffi(input: TokenStream) -> TokenStream {
    let hypervisor = parse_macro_input!(input as Path);

    TokenStream2::from_iter([generate_partition_ffi(hypervisor)]).into()
}
