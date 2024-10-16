use std::fmt::Display;
use std::str::FromStr;
use std::time::Duration;

use darling::{FromAttributes, FromMeta};
use proc_macro2::Ident;
use strum::{Display, EnumString};
use syn::spanned::Spanned;
use syn::{parse_quote, Attribute, Item, Type};

use crate::parse::util::{
    contains_attribute, remove_attributes, MayFromAttributes, WrappedByteSize, WrappedDuration,
};

#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub struct ApexName(String);

impl FromMeta for ApexName {
    fn from_string(value: &str) -> darling::Result<Self> {
        if !value.is_ascii() {
            Err(darling::Error::custom(
                "Port name contains not ASCII-printable characters",
            ))
        } else if value.len() > 16 {
            Err(darling::Error::custom(
                "Port name must be 16 ASCII characters or less wide",
            ))
        } else {
            Ok(Self(value.to_string()))
        }
    }
}

impl Display for ApexName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, EnumString, Display)]
#[strum(ascii_case_insensitive)]
pub enum QueuingDiscipline {
    Fifo,
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
    #[darling(default = "ApexName::default")]
    pub name: ApexName,
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
    #[darling(default = "ApexName::default")]
    pub name: ApexName,
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
    #[darling(default = "ApexName::default")]
    pub name: ApexName,
    pub msg_size: WrappedByteSize,
    pub msg_count: u32,
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
    #[darling(default = "ApexName::default")]
    pub name: ApexName,
    pub msg_size: WrappedByteSize,
    pub msg_count: u32,
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

#[derive(Debug, Clone, Display)]
pub enum Channel {
    SamplingOut(Ident, SamplingOutProc),
    SamplingIn(Ident, SamplingInProc),
    QueuingOut(Ident, QueuingOutProc),
    QueuingIn(Ident, QueuingInProc),
}

impl Channel {
    /// Used for identifying this channel in contexts and its `mod`
    pub fn ident(&self) -> Ident {
        match self {
            Channel::SamplingOut(ident, _) => ident.clone(),
            Channel::SamplingIn(ident, _) => ident.clone(),
            Channel::QueuingOut(ident, _) => ident.clone(),
            Channel::QueuingIn(ident, _) => ident.clone(),
        }
    }

    /// Solely used for the static name
    pub fn name(&self) -> ApexName {
        match self {
            Channel::SamplingOut(_ident, ch) => &ch.name,
            Channel::SamplingIn(_ident, ch) => &ch.name,
            Channel::QueuingOut(_ident, ch) => &ch.name,
            Channel::QueuingIn(_ident, ch) => &ch.name,
        }
        .clone()
    }

    pub fn msg_size(&self) -> u64 {
        match self {
            Channel::SamplingOut(_ident, ch) => ch.msg_size.bytes(),
            Channel::SamplingIn(_ident, ch) => ch.msg_size.bytes(),
            Channel::QueuingOut(_ident, ch) => ch.msg_size.bytes(),
            Channel::QueuingIn(_ident, ch) => ch.msg_size.bytes(),
        }
    }

    pub fn msg_count(&self) -> Option<u32> {
        match self {
            Channel::QueuingOut(_ident, ch) => Some(ch.msg_count),
            Channel::QueuingIn(_ident, ch) => Some(ch.msg_count),
            _ => None,
        }
    }

    pub fn discipline(&self) -> Option<QueuingDiscipline> {
        match self {
            Channel::QueuingOut(_ident, ch) => Some(ch.discipline),
            Channel::QueuingIn(_ident, ch) => Some(ch.discipline),
            _ => None,
        }
    }

    pub fn refresh_period(&self) -> Option<Duration> {
        if let Channel::SamplingIn(_ident, ch) = self {
            return Some(ch.refresh_period.into());
        }
        None
    }

    pub fn typ(&self) -> Type {
        match self {
            Channel::SamplingOut(_, s) => {
                let size = s.msg_size.bytes() as u32;
                parse_quote!(ConstSamplingPortSource::< #size , Hypervisor>)
            }
            Channel::SamplingIn(_, s) => {
                let size = s.msg_size.bytes() as u32;
                parse_quote!(ConstSamplingPortDestination::< #size , Hypervisor>)
            }
            Channel::QueuingOut(_, q) => {
                let size = q.msg_size.bytes() as u32;
                let count = q.msg_count;
                parse_quote!(ConstQueuingPortSender::< #size , #count , Hypervisor>)
            }
            Channel::QueuingIn(_, q) => {
                let size = q.msg_size.bytes() as u32;
                let count = q.msg_count;
                parse_quote!(ConstQueuingPortReceiver::< #size , #count , Hypervisor>)
            }
        }
    }

    pub fn from_content(items: &mut Vec<Item>) -> syn::Result<Vec<Channel>> {
        let mut channel = vec![];
        *items = items
            .drain(..)
            .filter_map(|item| match item {
                Item::Struct(mut item) => {
                    let mut vec: Vec<Option<darling::Result<Channel>>> = vec![
                        SamplingOutProc::may_from_attributes(&mut item.attrs).map(|x| {
                            let mut x = x?;
                            if x.name.0.is_empty() {
                                let alt_name: ApexName =
                                    FromMeta::from_string(&item.ident.to_string())?;
                                x.name = alt_name;
                            }
                            darling::Result::Ok(Channel::SamplingOut(item.ident.clone(), x))
                        }),
                        SamplingInProc::may_from_attributes(&mut item.attrs).map(|x| {
                            let mut x = x?;
                            if x.name.0.is_empty() {
                                let alt_name: ApexName =
                                    FromMeta::from_string(&item.ident.to_string())?;

                                x.name = alt_name;
                            }
                            darling::Result::Ok(Channel::SamplingIn(item.ident.clone(), x))
                        }),
                        QueuingOutProc::may_from_attributes(&mut item.attrs).map(|x| {
                            let mut x = x?;
                            if x.name.0.is_empty() {
                                let alt_name: ApexName =
                                    FromMeta::from_string(&item.ident.to_string())?;
                                x.name = alt_name;
                            }
                            darling::Result::Ok(Channel::QueuingOut(item.ident.clone(), x))
                        }),
                        QueuingInProc::may_from_attributes(&mut item.attrs).map(|x| {
                            let mut x = x?;
                            if x.name.0.is_empty() {
                                let alt_name: ApexName =
                                    FromMeta::from_string(&item.ident.to_string())?;
                                x.name = alt_name;
                            }
                            darling::Result::Ok(Channel::QueuingIn(item.ident.clone(), x))
                        }),
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

#[cfg(test)]
mod tests {
    use darling::FromMeta;

    use super::ApexName;

    #[test]
    fn long_port_name() {
        let port = "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";
        let port: darling::Result<ApexName> = FromMeta::from_string(port);
        assert!(port.is_err());
    }

    #[test]
    fn ascii_printable_name() {
        let port = "[AA!-=-14**\\";
        let port: darling::Result<ApexName> = FromMeta::from_string(port);
        assert!(port.is_ok());
    }

    #[test]
    fn non_ascii_printable_name() {
        let port = "\u{7FFF}";
        let port: darling::Result<ApexName> = FromMeta::from_string(port);
        assert!(port.is_err());
    }
}
