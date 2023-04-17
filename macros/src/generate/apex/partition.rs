use syn::{parse_quote, ItemImpl};

use crate::generate::context::Context;

impl Context {
    pub fn gen_partition(&self) -> ItemImpl {
        let ctx = self.get_context_ident();
        parse_quote! {
            impl<'a, H> #ctx <'a, H> {
                pub fn get_partition_status(&self) -> PartitionStatus {
                    Partition::get_partition_status()
                }
            }
        }
    }
}
