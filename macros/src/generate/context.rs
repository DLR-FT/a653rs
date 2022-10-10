use convert_case::{Case, Casing};
use quote::{format_ident, quote};
use strum::{Display, EnumIter, IntoEnumIterator};
use syn::parse::Parser;
use syn::{parse_quote, Field, Ident, Item, ItemImpl};

use crate::parse::process::{Process, SystemTime};
use crate::partition::Partition;

#[derive(Debug, Display, EnumIter, Clone, Copy)]
pub enum Context {
    Start,
    Periodic,
    Aperiodic,
}

impl Context {
    pub fn from_process(proc: &Process) -> Self {
        match proc.period {
            SystemTime::Infinite => Self::Aperiodic,
            SystemTime::Normal(_) => Self::Periodic,
        }
    }

    pub fn gen_all(part: &Partition) -> impl Iterator<Item = Item> + '_ {
        Self::gen_contexts(part)
            .map(Into::into)
            .chain(Self::gen_all_extensions().map(Into::into))
    }

    fn gen_contexts(part: &Partition) -> impl Iterator<Item = Item> + '_ {
        Self::gen_start_context()
            .chain(Self::Periodic.gen_process_context(part))
            .chain(Self::Aperiodic.gen_process_context(part))
    }

    fn gen_start_context() -> impl Iterator<Item = Item> {
        let name = Context::Start.get_context_ident();
        let st = parse_quote! {
            struct #name <'a, H> {
                _p: core::marker::PhantomData<H>,
                ctx: &'a mut apex_rs::prelude::StartContext<Hypervisor>,
            }
        };

        let im = parse_quote! {
            impl<'a, H> #name <'a, H>{
                pub fn new(ctx: &'a mut apex_rs::prelude::StartContext<Hypervisor>) -> Self {
                    Self {
                        _p: core::marker::PhantomData::default(),
                        ctx,
                    }
                }
            }
        };
        [st, im].into_iter()
    }

    fn gen_process_context(&self, part: &Partition) -> impl Iterator<Item = Item> {
        let name = self.get_context_ident();
        let fields = part.gen_context_fields();
        let field_names: Vec<_> = part
            .gen_context_fields()
            .map(|f| f.ident)
            .flatten()
            .collect();
        let st = parse_quote! {
            struct #name <'a, H> {
                _p: core::marker::PhantomData<H>,
                proc_self: &'a Process<Hypervisor>,
                #(#fields),*
            }
        };
        let im = parse_quote! {
            impl<'a, H> #name <'a, H>{
                pub fn new(proc_self: &'a Process<Hypervisor>) -> Self {
                    Self{
                        _p: core::marker::PhantomData::default(),
                        proc_self,
                        #(#field_names: unsafe{& #field_names::VALUE}),*
                    }
                }
            }
        };

        [st, im].into_iter()
    }

    fn gen_extension(&self) -> impl Iterator<Item = ItemImpl> {
        self.gen_time().chain(self.gen_error())
    }

    fn gen_all_extensions() -> impl Iterator<Item = ItemImpl> {
        Context::iter().map(|c| c.gen_extension()).flatten()
    }

    pub fn get_context_ident(&self) -> Ident {
        match self {
            Context::Start => parse_quote!(StartContext),
            Context::Periodic => parse_quote!(PeriodicContext),
            Context::Aperiodic => parse_quote!(AperiodicContext),
        }
    }
}

impl Partition {
    pub fn gen_context_process_fields(&self) -> impl Iterator<Item = Field> + '_ {
        self.processes.iter().map(|p| {
            let name = &p.name;
            Field::parse_named
                .parse2(quote!(#name: &'a Option<Process<Hypervisor>>))
                .unwrap()
        })
    }

    pub fn gen_context_channel_fields(&self) -> impl Iterator<Item = Field> + '_ {
        self.channel.iter().map(|c| {
            let name = &c.name();
            let name = format_ident!(
                "{}",
                name.to_string().to_case(Case::Snake),
                span = name.span()
            );
            let typ = c.typ();
            Field::parse_named
                .parse2(quote!(#name: &'a Option< #typ >))
                .unwrap()
        })
    }

    pub fn gen_context_fields(&self) -> impl Iterator<Item = Field> + '_ {
        self.gen_context_process_fields()
            .chain(self.gen_context_channel_fields())
    }
}
