use darling::util::Flag;
use darling::FromAttributes;
use proc_macro2::Span;
use strum::AsRefStr;
use syn::spanned::Spanned;
use syn::{Item, ItemFn};

use crate::parse::util::{no_return_type, remove_attributes, single_function_argument};

#[derive(Debug, Copy, Clone, AsRefStr)]
enum StartType {
    Warm(Span),
    Cold(Span),
}

impl StartType {
    fn span(&self) -> &Span {
        match self {
            StartType::Warm(s) => s,
            StartType::Cold(s) => s,
        }
    }
}

#[derive(Debug, Clone, FromAttributes)]
#[darling(attributes(start))]
struct StartFlags {
    warm: Flag,
    cold: Flag,
}

impl TryFrom<StartFlags> for Option<StartType> {
    type Error = syn::Error;

    fn try_from(value: StartFlags) -> Result<Self, Self::Error> {
        let mut flags = vec![];
        if value.warm.is_present() {
            flags.push(StartType::Warm(value.warm.span()))
        }
        if value.cold.is_present() {
            flags.push(StartType::Cold(value.cold.span()))
        }
        match flags.len() {
            0 => Ok(None),
            1 => Ok(Some(flags[0])),
            _ => {
                let mut flags = flags.iter();
                let mut err = syn::Error::new(
                    *flags.next().unwrap().span(),
                    "Multiple start flags attached to same function.",
                );
                for (i, flag) in flags.enumerate() {
                    err.combine(syn::Error::new(*flag.span(), format!("{}th flag", i + 2)))
                }
                Err(err)
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Start {
    warm: ItemFn,
    cold: ItemFn,
}

impl Start {
    pub fn warm(&self) -> &ItemFn {
        &self.warm
    }

    pub fn cold(&self) -> &ItemFn {
        &self.cold
    }

    fn verify_fn_form(self) -> syn::Result<Start> {
        single_function_argument(
            &syn::Type::Path(syn::parse_str("start::Context").unwrap()),
            &self.warm.sig,
        )?;
        no_return_type("WarmStart", &self.warm.sig.output)?;

        single_function_argument(
            &syn::Type::Path(syn::parse_str("start::Context").unwrap()),
            &self.cold.sig,
        )?;
        no_return_type("ColdStart", &self.cold.sig.output)?;

        Ok(self)
    }

    pub fn from_content(root: &Span, items: &mut [Item]) -> syn::Result<Start> {
        let mut warm: Option<ItemFn> = None;
        let mut cold: Option<ItemFn> = None;
        for item in items.iter_mut().filter_map(|item| match item {
            Item::Fn(f) => Some(f),
            _ => None,
        }) {
            let start = StartFlags::from_attributes(&item.attrs)?;
            let start: Option<StartType> = start.try_into()?;
            let start = if let Some(start) = start {
                start
            } else {
                continue;
            };

            // Remove start attributes from item //
            remove_attributes("start", &mut item.attrs)?;
            // Remove start attributes from item //

            let leftover = match start {
                StartType::Warm(_) => warm.replace(item.clone()),
                StartType::Cold(_) => cold.replace(item.clone()),
            };
            if let Some(leftover) = leftover {
                let mut err = syn::Error::new(
                    item.span(),
                    format!("{}Start already defined", start.as_ref()),
                );
                err.combine(syn::Error::new(leftover.span(), "First definition here"));
                return Err(err);
            }
        }
        Start {
            warm: warm
                .ok_or_else(|| syn::Error::new(root.span(), "No 'start(warm)' function defnied"))?,
            cold: cold
                .ok_or_else(|| syn::Error::new(root.span(), "No 'start(cold)' function defnied"))?,
        }
        .verify_fn_form()
    }
}
