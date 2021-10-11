use cep47::contract_utils::Dict;

use crate::{Commission, Gauge, Warehouse};

const GAUGES_DICT: &str = "gauges";
const WAREHOUSES_DICT: &str = "warehouses";
const COMMISSIONS_DICT: &str = "commissions";

pub struct Gauges {
    dict: Dict,
}

impl Gauges {
    pub fn instance() -> Gauges {
        Gauges {
            dict: Dict::instance(GAUGES_DICT),
        }
    }

    pub fn init() {
        Dict::init(GAUGES_DICT)
    }

    pub fn get(&self, key: &str) -> Option<Gauge> {
        self.dict.get(key)
    }

    pub fn set(&self, key: &str, value: Gauge) {
        self.dict.set(key, value);
    }

    pub fn remove(&self, key: &str) {
        self.dict.remove::<Gauge>(key);
    }
}

pub struct Warehouses {
    dict: Dict,
}

impl Warehouses {
    pub fn instance() -> Warehouses {
        Warehouses {
            dict: Dict::instance(WAREHOUSES_DICT),
        }
    }

    pub fn init() {
        Dict::init(WAREHOUSES_DICT)
    }

    pub fn get(&self, key: &str) -> Option<Warehouse> {
        self.dict.get(key)
    }

    pub fn set(&self, key: &str, value: Warehouse) {
        self.dict.set(key, value);
    }

    pub fn remove(&self, key: &str) {
        self.dict.remove::<Warehouse>(key);
    }
}

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
