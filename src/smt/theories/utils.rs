//! Macro helpers for defining theories

use smt::smt::SMTNode;

#[macro_export]
macro_rules! impl_smt_node {
    ($fns: ident) => {
        impl SMTNode for $fns {
            fn is_var(&self) -> bool {
                if let $fns::FreeVar(_) = *self {
                    true
                } else {
                    false
                }
            }

            fn is_const(&self) -> bool {
                if let $fns::Const(_,_) = *self {
                    true
                } else {
                    false
                }
            }
        }
    }
}
