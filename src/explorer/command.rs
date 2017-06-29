//! Defines Commands available to the Explorer.

use context::utils::{Key, to_key, convert_to_u64};

#[derive(Clone, Debug, PartialEq, Serialize)]
pub enum ValType {
    Symbolic,
    Concrete,
}

impl ValType {
    pub fn is_symbolic(&self) -> bool {
        match *self {
            ValType::Symbolic => true,
            _ => false,
        }
    }

    pub fn is_concrete(&self) -> bool {
        !self.is_symbolic() 
    }
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub enum Command {
    FollowTrue,
    FollowFalse,
    Continue,
    Step,
    Debug,
    Assertion,
    Run,
    Query,
    Help,
    Safety,
    Invalid,
    SetContext((Key, ValType)),
    SetVar((String, String)),
    Exit,
}

impl Command {
    pub fn is_invalid(&self) -> bool {
        *self == Command::Invalid
    }

    pub fn is_valid(&self) -> bool {
        !self.is_invalid()
    }

    pub fn is_set(&self) -> bool {
        match *self {
            Command::SetVar(_) | Command::SetContext(_) => true,
            _ => false,
        }
    }

    pub fn is_chainable(&self) -> bool {
        !self.is_invalid() && !self.is_set()
    }
}

/* Implementation notes:
 * Maybe have StepInto and StepOver later.
 * Reserve 'e' for environment variables.
 * Add method to set memory range as symbolic
 */
impl From<String> for Command {
    fn from(s: String) -> Command {
        let c = s.chars().nth(0).unwrap();
        match c {
            'T' => Command::FollowTrue,
            'F' => Command::FollowFalse,
            'C' => Command::Continue,
            'S' => Command::Step,
            'D' => Command::Debug,
            '?' => Command::Assertion,
            'Q' => Command::Query,
            'E' => {
                let (_, cmd) = s.split_at(2);
                let op: Vec<&str> = cmd.split("=").collect();

                let reg = to_key(op[0].trim().to_owned());

                if let Some(val) = convert_to_u64(op[1].trim().to_owned()) {
                    Command::SetContext((reg, ValType::Concrete))
                } else {
                    Command::SetContext((reg, ValType::Symbolic))
                }
            }
            'H' => Command::Help,
            'R' => Command::Run,
            'X' => Command::Safety,
            _ => Command::Invalid,
        }
    }
}
