use std::str::FromStr;
use std::time::Duration;

use bytesize::ByteSize;
use darling::{FromMeta, ToTokens};
use syn::{parse_quote, Attribute, FnArg, Ident, ReturnType, Signature};

pub fn contains_attribute(attr: &str, attrs: &[Attribute]) -> bool {
    attrs
        .iter()
        .flat_map(|a| a.parse_meta())
        .flat_map(|m| m.path().get_ident().cloned())
        .any(|i| i.to_string().eq(attr))
}

pub trait MayFromAttributes: Sized {
    fn may_from_attributes(attrs: &mut Vec<Attribute>) -> Option<darling::Result<Self>>;
}

///
/// Verify that no return type is specified in `output`  
/// `ident` is used for the error message only, declaring that no output is allowed for `ident`
pub fn no_return_type(ident: &str, output: &syn::ReturnType) -> syn::Result<()> {
    let empty: ReturnType = parse_quote! {-> ()};
    if empty.eq(output) {
        return Ok(());
    }
    if let syn::ReturnType::Type(_, _) = output {
        return Err(syn::Error::new_spanned(
            output.clone(),
            format!("{ident} outputs are not allowed"),
        ));
    }
    Ok(())
}

pub fn single_function_argument(ty: &syn::Type, sig: &Signature) -> syn::Result<()> {
    let path: String = ty
        .to_token_stream()
        .to_string()
        .split_whitespace()
        .collect();
    let msg = format!("A single input is expected: {path}");
    if let Some(FnArg::Typed(t)) = sig.inputs.first() {
        if !ty.eq(&t.ty) {
            return Err(syn::Error::new_spanned(t.ty.clone(), msg));
        }
    } else {
        return Err(syn::Error::new(sig.paren_token.span, msg));
    }
    if sig.inputs.len() > 1 {
        return Err(syn::Error::new(sig.paren_token.span, msg));
    }
    Ok(())
}

pub fn remove_attributes(attr: &str, attrs: &mut Vec<Attribute>) -> syn::Result<()> {
    let attr = syn::parse_str::<Ident>(attr)?;
    attrs.retain(|a| {
        a.path
            .segments
            .first()
            .map_or_else(|| true, |p| !p.ident.eq(&attr))
    });
    Ok(())
}

#[derive(Debug, Clone)]
pub struct WrappedByteSize(ByteSize);

impl WrappedByteSize {
    pub fn bytes(&self) -> u64 {
        self.0.as_u64()
    }
}

impl From<WrappedByteSize> for ByteSize {
    fn from(w: WrappedByteSize) -> Self {
        w.0
    }
}

impl FromMeta for WrappedByteSize {
    fn from_string(value: &str) -> darling::Result<Self> {
        match ByteSize::from_str(value) {
            Ok(s) => Ok(WrappedByteSize(s)),
            Err(e) => Err(darling::Error::unsupported_shape(&e)),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct WrappedDuration(Duration);

impl From<WrappedDuration> for Duration {
    fn from(dur: WrappedDuration) -> Self {
        dur.0
    }
}

impl FromMeta for WrappedDuration {
    fn from_string(value: &str) -> darling::Result<Self> {
        match humantime::parse_duration(value) {
            Ok(d) => Ok(WrappedDuration(d)),
            Err(e) => Err(darling::Error::unsupported_shape(&e.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::parse::util::*;

    #[test]
    fn test_contains_attribute() {
        use syn::{parse_quote, Attribute};

        let attr1: Attribute = parse_quote!(#[sample_attr]);
        let attr2: Attribute = parse_quote!(#[another_attr]);

        let attrs = vec![attr1, attr2];

        assert!(contains_attribute("sample_attr", &attrs));
        assert!(contains_attribute("another_attr", &attrs));
        assert!(!contains_attribute("non_existent_attr", &attrs));
    }

    #[test]
    fn test_no_return_type() {
        use syn::{parse_quote, ReturnType};

        let valid_return: ReturnType = parse_quote!(-> ());

        // Function with valid return type
        assert!(no_return_type("TestFn", &valid_return).is_ok());

        // Function with invalid return type (usize)
        let invalid_return: ReturnType = parse_quote!(-> usize);
        assert!(no_return_type("TestFn", &invalid_return).is_err());
    }

    #[test]
    fn test_remove_attributes() {
        use syn::{parse_quote, Attribute};

        let attr1: Attribute = parse_quote!(#[sample_attr]);
        let attr2: Attribute = parse_quote!(#[another_attr]);

        let mut attrs = vec![attr1, attr2.clone()];

        // Removing an attribute that exists
        remove_attributes("sample_attr", &mut attrs).unwrap();
        assert_eq!(attrs, vec![attr2.clone()]);

        // Removing an attribute that does not exist
        remove_attributes("non_existent_attr", &mut attrs).unwrap();
        assert_eq!(attrs, vec![attr2]);
    }
}
