use darling::util::Flag;
use darling::FromAttributes;
use proc_macro2::Span;
use strum::AsRefStr;
use syn::spanned::Spanned;
use syn::{Item, ItemFn};

use crate::parse::util::{no_return_type, remove_attributes, single_function_argument};

#[derive(Debug, Copy, Clone, AsRefStr)]
enum StartType {
    Warm,
    Cold,
}

#[derive(Debug, Clone, FromAttributes)]
#[darling(attributes(start))]
struct StartFlags {
    warm: Flag,
    cold: Flag,
}

impl From<StartFlags> for Vec<StartType> {
    fn from(value: StartFlags) -> Self {
        let mut flags = vec![];
        if value.warm.is_present() {
            flags.push(StartType::Warm)
        }
        if value.cold.is_present() {
            flags.push(StartType::Cold)
        }
        flags
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
            let starts: Vec<StartType> = start.into();
            for start in starts {
                // Remove start attributes from item
                remove_attributes("start", &mut item.attrs)?;

                let leftover = match start {
                    StartType::Warm => warm.replace(item.clone()),
                    StartType::Cold => cold.replace(item.clone()),
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

#[cfg(test)]
mod tests {
    use syn::parse_quote;

    use crate::parse::start::*;

    #[test]
    fn test_start_flags_from_attributes_both_present() {
        use darling::FromAttributes;

        // Mock attributes
        let warm_attr = parse_quote!(#[start(warm)]);
        let cold_attr = parse_quote!(#[start(cold)]);

        // Test case: Both warm and cold attributes are present
        let flags: StartFlags = StartFlags::from_attributes(&[warm_attr, cold_attr])
            .expect("Failed to parse StartFlags");
        assert!(flags.warm.is_present());
        assert!(flags.cold.is_present());
    }
}
