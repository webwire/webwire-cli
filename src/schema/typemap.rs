use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::{Rc, Weak};

use super::fqtn::FQTN;
use super::r#type::UserDefinedType;

#[derive(Default)]
pub struct TypeMap {
    map: HashMap<FQTN, Weak<RefCell<UserDefinedType>>>,
}

impl TypeMap {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }
    pub fn insert(&mut self, type_rc: &Rc<RefCell<UserDefinedType>>) {
        self.map
            .insert(type_rc.borrow().fqtn().clone(), Rc::downgrade(type_rc));
    }
    pub fn get(&self, fqtn: &FQTN) -> Option<Weak<RefCell<UserDefinedType>>> {
        match self.map.get(fqtn) {
            Some(type_rc) => Some(type_rc.clone()),
            None => None,
        }
    }
}
