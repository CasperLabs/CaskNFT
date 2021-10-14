use crate::Commission;
use cep47::contract_utils::Dict;

const COMMISSIONS_DICT: &str = "commissions";

pub struct Commissions {
    dict: Dict,
}

impl Commissions {
    pub fn instance() -> Commissions {
        Commissions {
            dict: Dict::instance(COMMISSIONS_DICT),
        }
    }

    pub fn init() {
        Dict::init(COMMISSIONS_DICT)
    }

    pub fn get(&self, key: &str) -> Option<Commission> {
        self.dict.get(key)
    }

    pub fn set(&self, key: &str, value: Commission) {
        self.dict.set(key, value);
    }

    pub fn remove(&self, key: &str) {
        self.dict.remove::<Commission>(key);
    }
}
