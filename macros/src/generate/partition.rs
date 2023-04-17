use syn::{parse_quote, ItemImpl, ItemMod, ItemStruct, ItemType};

use super::context::Context;
use crate::parse::channel::Channel;
use crate::parse::process::Process;
use crate::partition::Partition;

impl Partition {
    pub fn gen_type_alias(&self) -> ItemType {
        let hyp_name = &self.hypervisor;
        parse_quote!(type Hypervisor = #hyp_name; )
    }

    pub fn gen_proc_mods(&self) -> syn::Result<impl Iterator<Item = ItemMod> + '_> {
        Ok(self
            .processes
            .iter()
            .map(Process::gen_process_mod)
            .collect::<syn::Result<Vec<ItemMod>>>()?
            .into_iter())
    }

    pub fn gen_channel_mods(&self) -> syn::Result<impl Iterator<Item = ItemMod>> {
        Ok(self
            .channel
            .iter()
            .map(Channel::gen_channel_mod)
            .collect::<syn::Result<Vec<ItemMod>>>()?
            .into_iter())
    }

    pub fn gen_start_mod(&self) -> ItemMod {
        let ctx = Context::Start.get_context_ident();
        parse_quote! {
            mod start {
                use super::Hypervisor;
                pub(super) type Context<'a> = super:: #ctx <'a, Hypervisor>;
            }
        }
    }

    pub fn gen_struct(&self) -> ItemStruct {
        parse_quote! {
            pub struct Partition;
        }
    }

    pub fn gen_impl(&self) -> ItemImpl {
        let cold_start = &self.cold_start.sig.ident;
        let warm_start = &self.warm_start.sig.ident;
        parse_quote! {
            impl apex_rs::prelude::Partition<Hypervisor> for Partition{
                fn cold_start(&self, ctx: &mut apex_rs::prelude::StartContext<Hypervisor>){
                    let ctx = start::Context::new(ctx);
                    #cold_start (ctx)
                }

                fn warm_start(&self, ctx: &mut apex_rs::prelude::StartContext<Hypervisor>){
                    let ctx = start::Context::new(ctx);
                    #warm_start (ctx)
                }
            }
        }
    }
}
