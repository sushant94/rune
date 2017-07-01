//! Defines Commands available to the Explorer.

use context::utils::{Key, to_key, convert_to_u64, to_assignment, SAssignment, ValType};

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
    Save,
    Safety,
    Invalid,
    SetContext(SAssignment),
    SetVar(SAssignment),
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
 * Have r2 style self-documentation
 */
impl From<String> for Command {
    fn from(s: String) -> Command {
        match s.chars().nth(0).unwrap() {
            'T' => Command::FollowTrue,
            'F' => Command::FollowFalse,
            'c' => Command::Continue,
            's' => Command::Step,
            'b' => {
                let (_, addr) = s.split_at(2);
                if let Some(val) = convert_to_u64(addr.trim()) {
                    Command::SetContext(SAssignment {
                        lvalue: Key::Mem(val as usize),
                        rvalue: ValType::Break,
                    })
                } else {
                    Command::Invalid
                }
            }
            'D' => Command::Debug,
            '?' => Command::Assertion,
            'Q' => Command::Query,
            'E' => {
                let (_, cmd) = s.split_at(2);
                if let Some(val) = to_assignment(cmd) {
                    Command::SetContext(val)
                } else {
                    Command::Invalid
                }
            }
            'S' => Command::Save,
            'H' => Command::Help,
            'R' => Command::Run,
            'X' => Command::Safety,
            _ => Command::Invalid,
        }
    }
}
