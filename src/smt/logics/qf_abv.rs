use smt::theories::{array_ex, bitvec, core};
use smt::smt::{Logic, SMTNode};
use std::fmt::{Display, Debug};
use std::fmt;

define_sorts_for_logic!(QF_ABV_Sorts,
                        BV -> bitvec::Sorts,
                        Core -> core::Sorts,
                        ArrayEx -> array_ex::Sorts<QF_ABV_Sorts, QF_ABV_Sorts>
                        );

define_fns_for_logic!(QF_ABV_Fn,
                      BVOps -> bitvec::OpCodes,
                      CoreOps -> core::OpCodes,
                      ArrayOps -> array_ex::OpCodes
                      );

define_logic!(QF_ABV,
              QF_ABV_Fn,
              QF_ABV_Sorts,
              map { QF_ABV_Sorts::BV(_) => bitvec::OpCodes::FreeVar,
                  QF_ABV_Sorts::ArrayEx(_) => array_ex::OpCodes::FreeVar
              }
              );
