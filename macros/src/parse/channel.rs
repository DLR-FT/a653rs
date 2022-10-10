use std::str::FromStr;

use darling::{FromAttributes, FromMeta};
use proc_macro2::Ident;
use strum::{Display, EnumDiscriminants, EnumIter, EnumString};
// use strum::{Display, EnumString, EnumVariantNames, VariantNames};
use syn::{parse_quote, spanned::Spanned, Attribute, Item, Type};

use crate::parse::util::{
    contains_attribute, remove_attributes, MayFromAttributes, WrappedByteSize, WrappedDuration,
};

#[derive(Debug, Clone, Copy, PartialEq, EnumString, Display)]
pub enum QueuingDiscipline {
    FIFO,
    Priority,
}

impl FromMeta for QueuingDiscipline {
    fn from_string(value: &str) -> darling::Result<Self> {
        QueuingDiscipline::from_str(value)
            .map_err(|e| darling::Error::unsupported_shape(&e.to_string()))
    }
}

#[derive(Debug, Clone, FromAttributes)]
#[darling(attributes(sampling_out))]
pub struct SamplingOutProc {
    pub msg_size: WrappedByteSize,
}

impl MayFromAttributes for SamplingOutProc {
    fn may_from_attributes(attrs: &mut Vec<Attribute>) -> Option<darling::Result<Self>> {
        if !contains_attribute("sampling_out", attrs) {
            return None;
        }
        let port = Some(Self::from_attributes(attrs.as_slice()));
        Some(remove_attributes("sampling_out", attrs))?.ok();
        port
    }
}

#[derive(Debug, Clone, FromAttributes)]
#[darling(attributes(sampling_in))]
pub struct SamplingInProc {
    pub msg_size: WrappedByteSize,
    pub refresh_period: WrappedDuration,
}

impl MayFromAttributes for SamplingInProc {
    fn may_from_attributes(attrs: &mut Vec<Attribute>) -> Option<darling::Result<Self>> {
        if !contains_attribute("sampling_in", attrs) {
            return None;
        }
        let port = Some(Self::from_attributes(attrs));
        Some(remove_attributes("sampling_in", attrs))?.ok();
        port
    }
}

#[derive(Debug, Clone, FromAttributes)]
#[darling(attributes(queuing_out))]
pub struct QueuingOutProc {
    pub msg_size: WrappedByteSize,
    pub msg_count: usize,
    pub discipline: QueuingDiscipline,
}

impl MayFromAttributes for QueuingOutProc {
    fn may_from_attributes(attrs: &mut Vec<Attribute>) -> Option<darling::Result<Self>> {
        if !contains_attribute("queuing_out", attrs) {
            return None;
        }
        let port = Some(Self::from_attributes(attrs));
        Some(remove_attributes("queuing_out", attrs))?.ok();
        port
    }
}

#[derive(Debug, Clone, FromAttributes)]
#[darling(attributes(queuing_in))]
pub struct QueuingInProc {
    pub msg_size: WrappedByteSize,
    pub msg_count: usize,
    pub discipline: QueuingDiscipline,
}

impl MayFromAttributes for QueuingInProc {
    fn may_from_attributes(attrs: &mut Vec<Attribute>) -> Option<darling::Result<Self>> {
        if !contains_attribute("queuing_in", attrs) {
            return None;
        }
        let port = Some(Self::from_attributes(attrs));
        Some(remove_attributes("queuing_in", attrs))?.ok();
        port
    }
}

#[derive(Debug, Clone, Display, EnumDiscriminants)]
#[strum_discriminants(derive(EnumIter))]
pub enum Channel {
    SamplingOut(Ident, SamplingOutProc),
    SamplingIn(Ident, SamplingInProc),
    QueuingOut(Ident, QueuingOutProc),
    QueuingIn(Ident, QueuingInProc),
}

impl Channel {
    pub fn name(&self) -> Ident {
        match self {
            Channel::SamplingOut(name, _) => name.clone(),
            Channel::SamplingIn(name, _) => name.clone(),
            Channel::QueuingOut(name, _) => name.clone(),
            Channel::QueuingIn(name, _) => name.clone(),
        }
    }

    pub fn typ(&self) -> Type {
        match self {
            Channel::SamplingOut(_, s) => {
                let size = s.msg_size.bytes() as u32;
                parse_quote!(SamplingPortSource::< #size , Hypervisor>)
            }
            Channel::SamplingIn(_, s) => {
                let size = s.msg_size.bytes() as u32;
                parse_quote!(SamplingPortDestination::< #size , Hypervisor>)
            }
            Channel::QueuingOut(_, q) => {
                let size = q.msg_size.bytes() as u32;
                let count = q.msg_count as u32;
                parse_quote!(QueuingPortSender::< #size , #count , Hypervisor>)
            }
            Channel::QueuingIn(_, q) => {
                let size = q.msg_size.bytes() as u32;
                let count = q.msg_count as u32;
                parse_quote!(QueuingPortReceiver::< #size , #count , Hypervisor>)
            }
        }
    }

    pub fn from_content<'a>(items: &mut Vec<Item>) -> syn::Result<Vec<Channel>> {
        let mut channel = vec![];
        *items = items
            .drain(..)
            .filter_map(|item| match item {
                Item::Struct(mut item) => {
                    let mut vec: Vec<Option<darling::Result<Channel>>> = vec![
                        SamplingOutProc::may_from_attributes(&mut item.attrs)
                            .map(|x| x.map(|x| Channel::SamplingOut(item.ident.clone(), x))),
                        SamplingInProc::may_from_attributes(&mut item.attrs)
                            .map(|x| x.map(|x| Channel::SamplingIn(item.ident.clone(), x))),
                        QueuingOutProc::may_from_attributes(&mut item.attrs)
                            .map(|x| x.map(|x| Channel::QueuingOut(item.ident.clone(), x))),
                        QueuingInProc::may_from_attributes(&mut item.attrs)
                            .map(|x| x.map(|x| Channel::QueuingIn(item.ident.clone(), x))),
                    ];
                    let vec: Vec<_> = vec
                        .drain(..)
                        .flatten()
                        .map(|c| c.map_err(|e| syn::Error::from(e.with_span(&item.span()))))
                        .collect();
                    match vec.len() {
                        0 => Some(Ok(Item::Struct(item))),
                        1 => match vec[0].clone() {
                            Ok(ch) => {
                                channel.push(ch);
                                None
                            }
                            Err(e) => Some(Err(e)),
                        },
                        _ => Some(Err(syn::Error::new_spanned(
                            item.clone(),
                            "Multiple Channels defined on same struct",
                        ))),
                    }
                }
                item => Some(Ok(item)),
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(channel)
    }
}
