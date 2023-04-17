use std::str::FromStr;
use std::string::ToString;

use bytesize::ByteSize;
use darling::{FromAttributes, FromMeta};
use quote::format_ident;
use strum::{Display, EnumString, EnumVariantNames, VariantNames};
use syn::spanned::Spanned;
use syn::{Attribute, Ident, Item};

use crate::parse::util::{
    contains_attribute, no_return_type, remove_attributes, single_function_argument,
    MayFromAttributes, WrappedByteSize, WrappedDuration,
};

#[derive(Debug, Clone, Display)]
pub enum SystemTime {
    Infinite,
    Normal(WrappedDuration),
}

impl FromMeta for SystemTime {
    fn from_string(value: &str) -> darling::Result<Self> {
        // TODO better suggestion for misspells
        if value.chars().any(|c| c.is_numeric()) {
            return Ok(Self::Normal(WrappedDuration::from_string(value)?));
        }

        if SystemTime::Infinite.to_string().eq_ignore_ascii_case(value) {
            return Ok(SystemTime::Infinite);
        }

        Err(darling::Error::unknown_value(
            "Expected either 'Infinite' or '<number><unit>'",
        ))
    }
}

#[derive(Debug, Clone, Copy, Display, EnumString, EnumVariantNames)]
pub enum Deadline {
    Soft,
    Hard,
}

impl FromMeta for Deadline {
    fn from_string(value: &str) -> darling::Result<Self> {
        Deadline::from_str(value).map_err(|_| {
            darling::Error::unsupported_shape_with_expected(
                value,
                &format!("{:?}", Deadline::VARIANTS),
            )
        })
    }
}

#[derive(Debug, Clone, FromAttributes)]
#[darling(attributes(aperiodic))]
pub struct Aperiodic {
    #[darling(default = "String::default")]
    name: String,
    time_capacity: SystemTime,
    stack_size: WrappedByteSize,
    base_priority: i32,
    deadline: Deadline,
}

impl MayFromAttributes for Aperiodic {
    fn may_from_attributes(attrs: &mut Vec<Attribute>) -> Option<darling::Result<Self>> {
        if !contains_attribute("aperiodic", attrs) {
            return None;
        }
        let process = Some(Self::from_attributes(attrs));
        Some(remove_attributes("aperiodic", attrs))?.ok();
        process
    }
}

#[derive(Debug, Clone, FromAttributes)]
#[darling(attributes(periodic))]
pub struct Periodic {
    #[darling(default = "String::default")]
    name: String,
    time_capacity: SystemTime,
    period: WrappedDuration,
    stack_size: WrappedByteSize,
    base_priority: i32,
    deadline: Deadline,
}

impl MayFromAttributes for Periodic {
    fn may_from_attributes(attrs: &mut Vec<Attribute>) -> Option<darling::Result<Self>> {
        if !contains_attribute("periodic", attrs) {
            return None;
        }
        let process = Some(Self::from_attributes(attrs));
        Some(remove_attributes("periodic", attrs))?.ok();
        process
    }
}

#[derive(Debug, Clone)]
pub struct Process {
    /// Solely used for the static name
    pub name: Ident,
    /// Used for identifying this process in contexts and its `mod`
    pub ident: Ident,
    pub time_capacity: SystemTime,
    pub period: SystemTime,
    pub stack_size: ByteSize,
    pub base_priority: i32,
    pub deadline: Deadline,
}

impl Process {
    fn from_aperiodic(ident: Ident, a: Aperiodic) -> Self {
        let name = if a.name.is_empty() {
            ident.clone()
        } else {
            format_ident!("{}", a.name, span = ident.span())
        };
        Process {
            time_capacity: a.time_capacity,
            period: SystemTime::Infinite,
            stack_size: a.stack_size.into(),
            base_priority: a.base_priority,
            deadline: a.deadline,
            name,
            ident,
        }
    }

    fn from_periodic(ident: Ident, p: Periodic) -> Self {
        let name = if p.name.is_empty() {
            ident.clone()
        } else {
            format_ident!("{}", p.name, span = ident.span())
        };
        Process {
            time_capacity: p.time_capacity,
            period: SystemTime::Normal(p.period.into()),
            stack_size: p.stack_size.into(),
            base_priority: p.base_priority,
            deadline: p.deadline,
            name,
            ident,
        }
    }

    pub fn from_content<'a>(items: &mut Vec<Item>) -> syn::Result<Vec<Process>> {
        let mut procs = vec![];
        for item in items.iter_mut().filter_map(|item| match item {
            Item::Fn(f) => Some(f),
            _ => None,
        }) {
            let mut vec: Vec<Option<darling::Result<Process>>> = vec![
                Aperiodic::may_from_attributes(&mut item.attrs)
                    .map(|x| x.map(|a| Process::from_aperiodic(item.sig.ident.clone(), a))),
                Periodic::may_from_attributes(&mut item.attrs)
                    .map(|x| x.map(|p| Process::from_periodic(item.sig.ident.clone(), p))),
            ];
            let vec: Vec<_> = vec
                .drain(..)
                .flatten()
                .map(|c| c.map_err(|e| syn::Error::from(e.with_span(&item.span()))))
                .collect();
            let proc = match vec.len() {
                0 => continue,
                1 => Ok(vec[0].clone()?),
                _ => Err(syn::Error::new_spanned(
                    item.clone(),
                    "Multiple Channels defined on same struct",
                )),
            }?;

            single_function_argument(
                &syn::Type::Path(syn::parse_str(&format!("{}::Context", item.sig.ident)).unwrap()),
                &item.sig,
            )?;
            no_return_type("Process", &item.sig.output)?;

            procs.push(proc);
        }

        Ok(procs)
    }
}
