//! Defines macros to build Logics on top of theories.

use std::convert::Into;
use std::fmt;

use smt::smt::Logic;

#[macro_export]
macro_rules! define_for_logic {
    ($logic: ident, $($variant: ident -> $sort: ty),*) => {
        #[derive(Clone, PartialEq)]
        pub enum $logic {
            $(
                $variant($sort),
            )*
        }
        
        $(
            impl Into<$logic> for $sort {
                fn into(self) -> $logic {
                    $logic::$variant(self)
                }
            }
        )*

            impl fmt::Display for $logic {
                fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                    let s = match *self {
                        $(
                            $logic::$variant(ref op) => op.to_string(),
                         )*
                    };
                    write!(f, "{}", s)
                }
            }
    }
}

#[macro_export]
macro_rules! define_logic {
    ($logic: ident, $op: ident, $sorts: ty) => {
        #[derive(Clone, Copy, Debug)]
        pub struct $logic;

        impl fmt::Display for $logic {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "{}", stringify!($logic))
            }
        }

        impl Logic for $logic {
            type Fns = $op;
            type Sorts = $sorts;

            fn free_var<T: AsRef<str>>(name: T) -> Self::Fns {
                $op::FreeVar(name.into().cloned())
            }
        }
    }
}
