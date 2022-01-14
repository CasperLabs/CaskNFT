use casper_contract::{contract_api::storage, unwrap_or_revert::UnwrapOrRevert};
use casper_types::Key;
use cep47::contract_utils::{ContractContext, ContractStorage, Dict};

const MINTERS_DICT: &str = "minters";
pub trait MinterControl<Storage: ContractStorage>: ContractContext<Storage> {
    fn init(&mut self) {
        Minters::init();
    }

    fn revoke_minter(&mut self, address: Key) {
        Minters::instance().revoke_minter(&address);
    }

    fn add_minter(&mut self, address: Key) {
        Minters::instance().add_minter(&address);
    }

    fn is_minter(&self) -> bool {
        let caller = self.get_caller();
        Minters::instance().is_minter(&caller)
    }
}

struct Minters {
    dict: Dict,
}

impl Minters {
    pub fn instance() -> Minters {
        Minters {
            dict: Dict::instance(MINTERS_DICT),
        }
    }
    pub fn init() {
        storage::new_dictionary(MINTERS_DICT).unwrap_or_revert();
    }

    pub fn is_minter(&self, key: &Key) -> bool {
        self.dict.get_by_key::<()>(key).is_some()
    }

    pub fn add_minter(&self, key: &Key) {
        self.dict.set_by_key(key, ());
    }

    pub fn revoke_minter(&self, key: &Key) {
        self.dict.remove_by_key::<()>(key);
    }
}
