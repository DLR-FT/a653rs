use std::time::Duration;

use convert_case::{Case, Casing};
use quote::format_ident;
use syn::{
    parse_quote, Expr, ExprCall, Ident, ItemConst, ItemImpl, ItemMod, ItemStatic, LitByteStr,
    LitStr, Path,
};

use crate::parse::channel::Channel;

impl Channel {
    pub fn gen_snake_ident(&self) -> Ident {
        let name = self.ident();
        format_ident!(
            "{}",
            name.to_string().to_case(Case::Snake),
            span = name.span()
        )
    }

    pub fn gen_static_value(&self) -> ItemStatic {
        let ch_type = self.typ();
        parse_quote! {
            pub static mut VALUE : Option< #ch_type > = None;
        }
    }

    pub fn gen_create_fn(&self) -> ItemImpl {
        let name = self.gen_snake_ident();
        let create_name = format_ident!("create_{name}");
        let create_channel_call = self.gen_create_channel_call();
        parse_quote! {
            impl<'a> super:: StartContext<'a, Hypervisor> {
                pub fn #create_name(&mut self) -> Result<(), Error>{
                    use core::str::FromStr;
                    let channel = self.ctx. #create_channel_call ?;
                    // This is safe because during cold/warm start only one thread works
                    unsafe {
                        VALUE = Some( channel );
                    }
                    Ok(())
                }
            }
        }
    }

    pub fn gen_create_channel_call(&self) -> ExprCall {
        match self {
            Channel::SamplingOut(_, _) => {
                parse_quote!(create_const_sampling_port_source(NAME))
            }
            Channel::SamplingIn(_, ch) => {
                let dur: Duration = ch.refresh_period.into();
                let dur = dur.as_nanos() as u64;
                let dur: Expr = parse_quote!(core::time::Duration::from_nanos(#dur));
                parse_quote!(create_const_sampling_port_destination(NAME, #dur))
            }
            Channel::QueuingOut(_, ch) => {
                let disc: Path = ch.discipline.into();
                parse_quote!(create_queuing_port_sender(NAME, #disc))
            }
            Channel::QueuingIn(_, ch) => {
                let disc: Path = ch.discipline.into();
                parse_quote!(create_queuing_port_receiver(NAME, #disc))
            }
        }
    }

    pub fn gen_static_name(&self) -> syn::Result<ItemConst> {
        const LEN: usize = 32;
        let name = self.name().to_string();
        let len = name.bytes().len();
        if len > LEN {
            return Err(syn::Error::new_spanned(
                name,
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

    pub fn gen_channel_mod(&self) -> syn::Result<ItemMod> {
        let name = self.gen_snake_ident();
        let static_name = self.gen_static_name()?;
        let static_value = self.gen_static_value();
        let create_fn = self.gen_create_fn();
        Ok(parse_quote! {
            mod #name {
                use super::Hypervisor;
                use a653rs::prelude::*;

                #create_fn
                #static_name
                #static_value
            }
        })
    }
}
