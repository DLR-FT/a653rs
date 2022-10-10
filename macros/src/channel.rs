use std::str::FromStr;

use darling::{FromAttributes, FromMeta};
use proc_macro2::Ident;
use strum::{Display, EnumDiscriminants, EnumIter, EnumString};
// use strum::{Display, EnumString, EnumVariantNames, VariantNames};
use syn::{spanned::Spanned, Attribute, ItemStruct};

use crate::util::{
    contains_attribute, remove_attributes, MayFromAttributes, WrappedByteSize, WrappedDuration,
};

#[derive(Debug, Clone, PartialEq, EnumString)]
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
pub struct SamplingOut {
    msg_size: WrappedByteSize,
}

impl MayFromAttributes for SamplingOut {
    fn may_from_attributes(attrs: &mut Vec<Attribute>) -> Option<darling::Result<Self>> {
        if !contains_attribute("sampling_out", attrs) {
            return None;
        }
        let port = Some(Self::from_attributes(attrs));
        Some(remove_attributes("sampling_out", attrs))?.ok();
        port
    }
}

impl From<SamplingOut> for Channel {
    fn from(s: SamplingOut) -> Self {
        Channel::SamplingOut(s)
    }
}

#[derive(Debug, Clone, FromAttributes)]
#[darling(attributes(sampling_in))]
pub struct SamplingIn {
    msg_size: WrappedByteSize,
    refresh_period: WrappedDuration,
}

impl MayFromAttributes for SamplingIn {
    fn may_from_attributes(attrs: &mut Vec<Attribute>) -> Option<darling::Result<Self>> {
        if !contains_attribute("sampling_in", attrs) {
            return None;
        }
        let port = Some(Self::from_attributes(attrs));
        Some(remove_attributes("sampling_in", attrs))?.ok();
        port
    }
}

impl From<SamplingIn> for Channel {
    fn from(s: SamplingIn) -> Self {
        Channel::SamplingIn(s)
    }
}

#[derive(Debug, Clone, FromAttributes)]
#[darling(attributes(queuing_out))]
pub struct QueuingOut {
    msg_size: WrappedByteSize,
    msg_count: usize,
    discipline: QueuingDiscipline,
}

impl MayFromAttributes for QueuingOut {
    fn may_from_attributes(attrs: &mut Vec<Attribute>) -> Option<darling::Result<Self>> {
        if !contains_attribute("queuing_out", attrs) {
            return None;
        }
        let port = Some(Self::from_attributes(attrs));
        Some(remove_attributes("queuing_out", attrs))?.ok();
        port
    }
}

impl From<QueuingOut> for Channel {
    fn from(s: QueuingOut) -> Self {
        Channel::QueuingOut(s)
    }
}

#[derive(Debug, Clone, FromAttributes)]
#[darling(attributes(queuing_in))]
pub struct QueuingIn {
    msg_size: WrappedByteSize,
    msg_count: usize,
    discipline: QueuingDiscipline,
}

impl MayFromAttributes for QueuingIn {
    fn may_from_attributes(attrs: &mut Vec<Attribute>) -> Option<darling::Result<Self>> {
        if !contains_attribute("queuing_in", attrs) {
            return None;
        }
        let port = Some(Self::from_attributes(attrs));
        Some(remove_attributes("queuing_in", attrs))?.ok();
        port
    }
}

impl From<QueuingIn> for Channel {
    fn from(s: QueuingIn) -> Self {
        Channel::QueuingIn(s)
    }
}

#[derive(Debug, Clone, Display, EnumDiscriminants)]
#[strum_discriminants(derive(EnumIter))]
pub enum Channel {
    SamplingOut(SamplingOut),
    SamplingIn(SamplingIn),
    QueuingOut(QueuingOut),
    QueuingIn(QueuingIn),
}

impl Channel {
    pub fn from_structs<'a>(items: &mut [&mut ItemStruct]) -> syn::Result<Vec<(Ident, Channel)>> {
        // let channel = SamplingOut::from_attributes(&a.attrs).unwrap();
        let mut channel = vec![];
        for item in items {
            // let item = *item;
            let mut vec: Vec<Option<darling::Result<Channel>>> = vec![
                SamplingOut::may_from_attributes(&mut item.attrs).map(|x| x.map(Channel::from)),
                SamplingIn::may_from_attributes(&mut item.attrs).map(|x| x.map(Channel::from)),
                QueuingOut::may_from_attributes(&mut item.attrs).map(|x| x.map(Channel::from)),
                QueuingIn::may_from_attributes(&mut item.attrs).map(|x| x.map(Channel::from)),
            ];
            let vec: Vec<_> = vec
                .drain(..)
                .flatten()
                .map(|c| c.map_err(|e| syn::Error::from(e.with_span(&item.span()))))
                .collect();
            let ch = match vec.len() {
                0 => continue,
                1 => Ok(vec[0].clone()?),
                _ => Err(syn::Error::new_spanned(
                    item.clone(),
                    "Multiple Channels defined on same struct",
                )),
            }?;
            // item.attrs
            channel.push((item.ident.clone(), ch));
        }
        Ok(channel)
    }
}

// impl TryFrom<ItemStruct> for Channel {
//     type Error = Option<syn::Error>;

//     fn try_from(item: ItemStruct) -> Result<Self, Self::Error> {
//         let attributes = item
//             .attrs
//             .iter()
//             .filter(|f| f.path.get_ident().is_some())
//             .filter(|f| {
//                 Channel::VARIANTS.contains(&f.path.get_ident().unwrap().to_string().as_str())
//             })
//             .collect::<Vec<_>>();
//         let attr = match attributes.len() {
//             0 => return Err(None),
//             1 => attributes[0],
//             _ => {
//                 return Err(Some(syn::Error::new_spanned(
//                     item,
//                     "Only a single channel attribute is supported at the moment",
//                 )))
//             }
//         };
//         let meta = attr.parse_meta()?;
//         let meta_list = match meta {
//             Meta::List(list) => list,
//             _ => {
//                 return Err(Some(syn::Error::new_spanned(
//                     meta,
//                     "expected a list-syle attribute",
//                 )))
//             }
//         };

//         panic!("A {meta_list:#?}")
//     }
// }
