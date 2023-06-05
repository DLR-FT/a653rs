use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::spanned::Spanned;
use syn::{parse_quote, Ident, Item, ItemFn, ItemMod, TypePath};

use crate::generate::context::Context;
use crate::parse::channel::Channel;
use crate::parse::process::Process;
use crate::parse::start::Start;

#[derive(Debug, Clone)]
pub struct Partition {
    pub name: Ident,
    pub hypervisor: TypePath,
    pub channel: Vec<Channel>,
    pub cold_start: ItemFn,
    pub warm_start: ItemFn,
    // pub processes: Vec<(ItemFn, Process)>,
    pub processes: Vec<Process>,
}

impl Partition {
    fn from_mod(hypervisor: TypePath, input: &mut ItemMod) -> syn::Result<Partition> {
        let root_span = input.span();
        let (_, content) = input.content.as_mut().unwrap();
        let channel = Channel::from_content(content)?;
        let start = Start::from_content(&root_span, content)?;
        let processes = Process::from_content(content)?;

        Ok(Partition {
            name: input.ident.clone(),
            hypervisor,
            channel,
            cold_start: start.cold().clone(),
            warm_start: start.warm().clone(),
            processes,
        })
    }

    pub fn expand_partition(mut input: ItemMod, hypervisor: TypePath) -> syn::Result<TokenStream> {
        // Parse user provided partition
        let part = Partition::from_mod(hypervisor, &mut input)?;
        let content: &mut Vec<Item> = &mut input.content.as_mut().unwrap().1;

        content.push(parse_quote!(
            use a653rs::prelude::*;
        ));
        content.push(part.gen_type_alias().into());
        content.push(part.gen_start_mod().into());
        content.push(part.gen_struct().into());
        content.push(part.gen_impl().into());
        content.extend(part.gen_proc_mods()?.map(Into::into));
        content.extend(part.gen_channel_mods()?.map(Into::into));
        content.extend(Context::gen_all(&part));

        let token_stream = input.to_token_stream();
        Ok(token_stream)
    }
}

///////////////////////////

// fn get_name_attr(attr: &Attribute) -> syn::Result<Option<Ident>> {
//     let meta = attr.parse_meta()?;
//     let meta_list = match meta {
//         Meta::List(list) => list,
//         _ => {
//             return Err(syn::Error::new_spanned(
//                 meta,
//                 "expected a list-syle attribute",
//             ))
//         }
//     };

//     let nested = match meta_list.nested.len() {
//         0 => return Ok(None),
//         1 => &meta_list.nested[0],
//         _ => {
//             return Err(syn::Error::new_spanned(
//                 meta_list.nested,
//                 "currently only a single getter attribute is supported",
//             ))
//         }
//     };

//     let name_value = match nested {
//         NestedMeta::Meta(Meta::NameValue(nv)) => nv,
//         _ => {
//             return Err(syn::Error::new_spanned(
//                 nested,
//                 "expected `name = \"<value>\"`",
//             ))
//         }
//     };

//     if !name_value.path.is_ident("name") {
//         return Err(syn::Error::new_spanned(
//             &name_value.path,
//             "unsupported getter attribute, expected `name`",
//         ));
//     }

//     match &name_value.lit {
//         Lit::Str(s) => syn::parse_str(&s.value()).map_err(|e| syn::Error::new_spanned(s, e)),
//         lit => Err(syn::Error::new_spanned(lit, "expected string literal")),
//     }
// }

// pub fn expand_getters(input: DeriveInput) -> syn::Result<TokenStream> {
//     let input2 = input.clone();
//     let fields = match input.data {
//         Data::Struct(DataStruct {
//             fields: Fields::Named(fields),
//             ..
//         }) => fields.named,
//         data => {
//             return Err(syn::Error::new_spanned(
//                 input2,
//                 "Only works for named field structs",
//             ));
//         }
//     };

//     let getters = fields
//         .into_iter()
//         .map(|f| {
//             let attrs: Vec<_> = f
//                 .attrs
//                 .iter()
//                 .filter(|attr| attr.path.is_ident("getter"))
//                 .collect();
//             let name_from_attr = match attrs.len() {
//                 0 => None,
//                 1 => get_name_attr(attrs[0])?,
//                 _ => {
//                     let mut error =
//                         syn::Error::new_spanned(attrs[1], "redundant `getter(name)` attribute");
//                     error.combine(syn::Error::new_spanned(attrs[0], "note: first one here"));
//                     return Err(error);
//                 }
//             };

//             let method_name =
//                 name_from_attr.unwrap_or_else(|| f.ident.clone().expect("a named field"));
//             let field_name = f.ident;
//             let field_ty = f.ty;

//             Ok(quote! {
//                 pub fn #method_name(&self) -> &#field_ty {
//                     &self.#field_name
//                 }
//             })
//         })
//         .collect::<syn::Result<TokenStream>>()?;

//     let st_name = input.ident;
//     let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

//     Ok(quote! {
//         #[automatically_derived]
//         impl #impl_generics #st_name #ty_generics #where_clause {
//             #getters
//         }
//     })
// }
