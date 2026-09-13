/*

Custom API calls

*/

use std::{collections::HashMap, hash::Hash};

#[derive(Eq)]
pub enum Print {
    Byte(u8),
    Wild,
}
impl PartialEq for Print {
    fn eq(&self, other: &Self) -> bool {
        match self {
            Print::Byte(b) => match other {
                Print::Byte(ob) => b == ob,
                Print::Wild => true,
            },
            Print::Wild => true,
        }
    }
}
impl Hash for Print {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            Print::Byte(v) => v.hash(state),
            Print::Wild => "wildcard".hash(state),
        }
    }
}

#[derive(Default)]
pub struct Fingerprint {
    pub next: HashMap<Print, Self>,
    pub name: Option<String>,
}
