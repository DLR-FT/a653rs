use proc_macro2::TokenStream;
use quote::format_ident;
use syn::{
    parse_quote, ExprPath, ItemConst, ItemFn, ItemImpl, ItemMod, ItemStatic, LitByteStr, LitStr,
};

use super::context::Context;
use crate::parse::process::Process;

impl Process {
    fn gen_wrapper_fn(&self) -> ItemFn {
        let ident = &self.ident;
        parse_quote! {
            pub(super) extern "C" fn wrapper () {
                let proc_self;
                unsafe {
                    proc_self = VALUE.as_ref().unwrap();
                }
                let ctx = Context::new(proc_self);
                super:: #ident(ctx)
            }
        }
    }

    pub fn gen_create_fn(&self) -> ItemImpl {
        let create_ident = format_ident!("create_{}", self.ident);
        let deadline: ExprPath = self.deadline.into();
        let priority = self.base_priority;
        let stack_size = self.stack_size.as_u64() as u32;
        let time_capacity: TokenStream = self.time_capacity.clone().into();
        let period: TokenStream = self.period.clone().into();
        parse_quote! {
            impl<'a> super:: StartContext<'a, Hypervisor> {
                pub fn #create_ident<'b>(&'b mut self) -> Result<&'b Process::<Hypervisor>, Error>{
                    use core::str::FromStr;
                    let attr =  ProcessAttribute {
                        period: #period,
                        time_capacity: #time_capacity,
                        entry_point: wrapper,
                        stack_size: #stack_size,
                        base_priority: #priority,
                        deadline: #deadline,
                        name: NAME,
                    };
                    let process = self.ctx.create_process(attr)?;
                    // This is safe because during cold/warm start only one thread works
                    unsafe {
                        VALUE = Some( process );
                        Ok(VALUE.as_ref().unwrap())
                    }
                }
            }
        }
    }

    pub fn gen_static_value(&self) -> ItemStatic {
        parse_quote! {
            pub(super) static mut VALUE : Option<Process::<Hypervisor>> = None;
        }
    }

    pub fn gen_static_name(&self) -> syn::Result<ItemConst> {
        const LEN: usize = 32;
        let name = self.name.to_string();
        let len = name.bytes().len();
        if len > LEN {
            return Err(syn::Error::new_spanned(
                self.name.clone(),
                format!("max name length is {LEN} bytes"),
            ));
        }
        let name = &format!("{name}{:\0<1$}", "", LEN - len);
        let lit_name: LitStr = parse_quote!(#name);
        let name = LitByteStr::new(name.as_bytes(), lit_name.span());

        Ok(parse_quote! {
             pub(super) const NAME: Name =
                Name::new( * #name );
        })
    }

    pub fn gen_process_mod(&self) -> syn::Result<ItemMod> {
        let ident = &self.ident;
        let wrapper = self.gen_wrapper_fn();
        let static_name = self.gen_static_name()?;
        let static_value = self.gen_static_value();
        let create_fn = self.gen_create_fn();
        let context_ident = Context::from_process(self).get_context_ident();
        Ok(parse_quote! {
            mod #ident {
                use a653rs::prelude::*;
                use super::Hypervisor;

                pub(super) type Context<'a> = super:: #context_ident <'a, Hypervisor> ;

                #wrapper
                #create_fn
                #static_name
                #static_value
            }
        })
    }
}
