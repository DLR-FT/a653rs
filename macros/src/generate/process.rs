use quote::format_ident;
use syn::{
    parse_quote, Expr, ItemConst, ItemFn, ItemImpl, ItemMod, ItemStatic, LitByteStr, LitStr, Path,
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
        parse_quote! {
            impl<'a> super:: StartContext<'a, Hypervisor> {
                pub fn #create_ident<'b>(&'b mut self) -> Result<&'b Process::<Hypervisor>, Error>{
                    use core::str::FromStr;
                    let attr =  ProcessAttribute {
                        period: PERIOD,
                        time_capacity: TIME_CAPACITY,
                        entry_point: wrapper,
                        stack_size: STACK_SIZE,
                        base_priority: BASE_PRIORITY,
                        deadline: DEADLINE,
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

    pub fn gen_consts(&self) -> syn::Result<Vec<ItemConst>> {
        Ok(vec![
            self.gen_const_name()?,
            self.gen_const_time_capacity(),
            self.gen_const_period(),
            self.gen_const_stack_size(),
            self.gen_const_base_priority(),
            self.gen_const_deadline(),
        ])
    }

    pub fn gen_const_name(&self) -> syn::Result<ItemConst> {
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

    pub fn gen_const_time_capacity(&self) -> ItemConst {
        let time_capacity: Expr = self.time_capacity.into();
        parse_quote! {
             pub(super) const TIME_CAPACITY: SystemTime = #time_capacity ;
        }
    }

    pub fn gen_const_period(&self) -> ItemConst {
        let period: Expr = self.period.into();
        parse_quote! {
             pub(super) const PERIOD: SystemTime = #period ;
        }
    }

    pub fn gen_const_stack_size(&self) -> ItemConst {
        let stack_size = self.stack_size.as_u64() as u32;
        parse_quote! {
             pub(super) const STACK_SIZE: StackSize = #stack_size ;
        }
    }

    pub fn gen_const_base_priority(&self) -> ItemConst {
        let base_priority = self.base_priority;
        parse_quote! {
             pub(super) const BASE_PRIORITY: Priority = #base_priority ;
        }
    }

    pub fn gen_const_deadline(&self) -> ItemConst {
        let deadline: Path = self.deadline.into();
        parse_quote! {
             pub(super) const DEADLINE: Deadline = #deadline ;
        }
    }

    pub fn gen_process_mod(&self) -> syn::Result<ItemMod> {
        let ident = &self.ident;
        let wrapper = self.gen_wrapper_fn();
        let consts = self.gen_consts()?;
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
                #(#consts)*
                #static_value
            }
        })
    }
}
