use std::time::Duration;

use proc_macro2::TokenStream;
use quote::format_ident;
use syn::{parse_quote, ExprPath, Path};

use crate::parse::channel::QueuingDiscipline;
use crate::parse::process::{Deadline, SystemTime};

impl From<Deadline> for ExprPath {
    fn from(d: Deadline) -> Self {
        let deadline = format_ident!("{}", d.to_string());
        parse_quote! {
             Deadline:: #deadline
        }
    }
}

impl From<SystemTime> for TokenStream {
    fn from(time: SystemTime) -> TokenStream {
        match time {
            SystemTime::Infinite => parse_quote!(SystemTime::Infinite),
            SystemTime::Normal(dur) => {
                let dur: Duration = dur.into();
                let dur = dur.as_nanos() as u64;
                parse_quote!(SystemTime::Normal(core::time::Duration::from_nanos(
                    #dur
                )))
            }
        }
    }
}

impl From<QueuingDiscipline> for Path {
    fn from(disc: QueuingDiscipline) -> Self {
        let var = format_ident!("{}", disc.to_string());
        parse_quote!(QueuingDiscipline:: #var)
    }
}
