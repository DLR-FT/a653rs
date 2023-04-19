use partition::Partition;
use proc_macro::TokenStream;
use syn::{parse_macro_input, ItemMod, TypePath};

mod generate;
mod parse;
mod partition;

/// TODO documentation
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
