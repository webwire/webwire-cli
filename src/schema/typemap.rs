use std::collections::HashMap;

use super::fqtn::FQTN;
use super::r#type::UserDefinedType;

pub struct TypeMap {
    map: HashMap<FQTN, UserDefinedType>,
}

impl TypeMap {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }
    pub fn insert(&mut self, ud_type: &UserDefinedType) {
        self.map.insert(ud_type.fqtn().clone(), ud_type.clone());
    }
    pub fn get(&self, fqtn: &FQTN) -> Option<&UserDefinedType> {
        self.map.get(fqtn)
    }
}
