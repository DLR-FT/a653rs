use syn::{parse_quote, ItemFn, ItemImpl};

use crate::generate::context::Context;

impl Context {
    pub fn gen_time(&self) -> impl Iterator<Item = ItemImpl> {
        [self.gen_time_p1(), self.gen_time_p4()].into_iter()
    }

    pub fn gen_time_p1(&self) -> ItemImpl {
        // TODO filter depending on start/periodic/aperiodic context
        let ctx = self.get_context_ident();
        let functions = vec![self.gen_time_timed_wait(), self.gen_time_replenish()];
        parse_quote! {
            impl<'a, Hypervisor: ApexTimeP1Ext> #ctx <'a, Hypervisor> {
                #(#functions)*
            }
        }
    }

    pub fn gen_time_p4(&self) -> ItemImpl {
        // TODO filter depending on start/periodic/aperiodic context
        let ctx = self.get_context_ident();
        let mut functions = vec![self.gen_time_get_time()];
        if matches!(self, Context::Periodic) {
            functions.push(self.gen_time_periodic_wait())
        }
        parse_quote! {
            impl<'a, Hypervisor: ApexTimeP4Ext> #ctx <'a, Hypervisor> {
                #(#functions)*
            }
        }
    }

    pub fn gen_time_timed_wait(&self) -> ItemFn {
        parse_quote! {
            pub fn timed_wait(&self, delay_time: core::time::Duration) -> Result<(), Error> {
                <Hypervisor as ApexTimeP1Ext>::timed_wait(delay_time)
            }
        }
    }

    pub fn gen_time_replenish(&self) -> ItemFn {
        parse_quote! {
            pub fn replenish(
                &self,
                budget_time: core::time::Duration,
            ) -> Result<(), Error> {
                <Hypervisor as ApexTimeP1Ext>::replenish(budget_time)
            }
        }
    }

    pub fn gen_time_periodic_wait(&self) -> ItemFn {
        parse_quote! {
            pub fn periodic_wait(&self) -> Result<(), Error> {
                <Hypervisor as ApexTimeP4Ext>::periodic_wait()
            }
        }
    }

    pub fn gen_time_get_time(&self) -> ItemFn {
        parse_quote! {
            pub fn get_time(&self) -> SystemTime {
                <Hypervisor as ApexTimeP4Ext>::get_time()
            }
        }
    }
}
