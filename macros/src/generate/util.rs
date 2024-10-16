use std::time::Duration;

use quote::format_ident;
use syn::{parse_quote, Expr, Path};

use crate::parse::channel::QueuingDiscipline;
use crate::parse::process::{Deadline, SystemTime};

impl From<Deadline> for Path {
    fn from(d: Deadline) -> Self {
        let deadline = format_ident!("{}", d.to_string());
        parse_quote! {
             Deadline:: #deadline
        }
    }
}

impl From<SystemTime> for Expr {
    fn from(time: SystemTime) -> Expr {
        match time {
            SystemTime::Infinite => parse_quote!(SystemTime::Infinite),
            SystemTime::Normal(dur) => {
                let dur: Duration = dur.into();
                let secs = dur.as_secs();
                let nanos = dur.subsec_nanos();
                parse_quote!(SystemTime::Normal(core::time::Duration::new( #secs , #nanos )))
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
