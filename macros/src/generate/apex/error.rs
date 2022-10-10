use syn::{parse_quote, ItemFn, ItemImpl};

use crate::generate::context::Context;

impl Context {
    pub fn gen_error(&self) -> impl Iterator<Item = ItemImpl> {
        [
            // self.gen_error_p1(),
            self.gen_error_p4(),
        ]
        .into_iter()
    }

    pub fn gen_error_p4(&self) -> ItemImpl {
        // TODO filter depending on start/periodic/aperiodic context
        let ctx = self.get_context_ident();
        let functions = vec![
            self.gen_error_report_application_message(),
            self.gen_error_raise_application_error(),
        ];
        parse_quote! {
            impl<'a, Hypervisor: ApexErrorP4Ext> #ctx <'a, Hypervisor> {
                #(#functions)*
            }
        }
    }

    pub fn gen_error_report_application_message(&self) -> ItemFn {
        parse_quote! {
            pub fn report_application_message(&self, msg: &[ApexByte]) -> Result<(), Error> {
                <Hypervisor as ApexErrorP4Ext>::report_application_message(msg)
            }
        }
    }

    pub fn gen_error_raise_application_error(&self) -> ItemFn {
        parse_quote! {
            pub fn raise_application_error(&self, msg: &[ApexByte]) -> Result<(), Error> {
                <Hypervisor as ApexErrorP4Ext>::raise_application_error(msg)
            }
        }
    }

    // TODO ErrorP1
}
