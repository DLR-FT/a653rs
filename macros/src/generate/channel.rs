use convert_case::{Case, Casing};
use quote::format_ident;
use syn::{
    parse_quote, ExprCall, Ident, ItemConst, ItemImpl, ItemMod, ItemStatic, LitByteStr, LitStr,
    Path,
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
            Channel::SamplingIn(_, _) => {
                parse_quote!(create_const_sampling_port_destination(NAME, REFRESH_PERIOD))
            }
            Channel::QueuingOut(_, _) => {
                parse_quote!(create_const_queuing_port_sender(NAME, DISCIPLINE))
            }
            Channel::QueuingIn(_, _) => {
                parse_quote!(create_const_queuing_port_receiver(NAME, DISCIPLINE))
            }
        }
    }

    pub fn gen_consts(&self) -> syn::Result<Vec<ItemConst>> {
        let mut consts = vec![self.gen_const_name()?, self.gen_const_msg_size()];
        if let Some(msg_count) = self.gen_const_msg_count() {
            consts.push(msg_count);
        }
        if let Some(discipline) = self.gen_const_discipline() {
            consts.push(discipline);
        }
        if let Some(refresh_period) = self.gen_const_refresh_period() {
            consts.push(refresh_period);
        }
        Ok(consts)
    }

    pub fn gen_const_name(&self) -> syn::Result<ItemConst> {
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

    pub fn gen_const_msg_size(&self) -> ItemConst {
        let msg_size = self.msg_size() as u32;
        parse_quote! {
             pub(super) const MSG_SIZE: MessageSize = #msg_size;
        }
    }

    pub fn gen_const_msg_count(&self) -> Option<ItemConst> {
        let msg_count = self.msg_count()?;
        Some(parse_quote! {
             pub(super) const NB_MSGS: MessageRange = #msg_count;
        })
    }

    pub fn gen_const_discipline(&self) -> Option<ItemConst> {
        let discipline: Path = self.discipline()?.into();
        Some(parse_quote! {
             pub(super) const DISCIPLINE: QueuingDiscipline = #discipline;
        })
    }

    pub fn gen_const_refresh_period(&self) -> Option<ItemConst> {
        let refresh_period = self.refresh_period()?;
        let secs = refresh_period.as_secs();
        let nanos = refresh_period.subsec_nanos();
        Some(parse_quote! {
             pub(super) const REFRESH_PERIOD: core::time::Duration =
                 core::time::Duration::new( #secs , #nanos );
        })
    }

    pub fn gen_channel_mod(&self) -> syn::Result<ItemMod> {
        let name = self.gen_snake_ident();
        let consts = self.gen_consts()?;
        let static_value = self.gen_static_value();
        let create_fn = self.gen_create_fn();
        Ok(parse_quote! {
            mod #name {
                use super::Hypervisor;
                use a653rs::prelude::*;

                #create_fn
                #(#consts)*
                #static_value
            }
        })
    }
}
