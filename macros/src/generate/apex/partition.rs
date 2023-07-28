use syn::{parse_quote, ItemImpl};

use crate::generate::context::Context;

impl Context {
    pub fn gen_partition(&self) -> ItemImpl {
        let ctx = self.get_context_ident();
        parse_quote! {
            impl<'a, H> #ctx <'a, H> {
                pub fn get_partition_status(&self) -> a653rs::prelude::PartitionStatus {
                    Partition::get_status()
                }

                pub fn set_partition_mode(&self, mode: a653rs::prelude::OperatingMode) -> Result<(), Error> {
                    Partition::set_mode(mode)
                }
            }
        }
    }
}
