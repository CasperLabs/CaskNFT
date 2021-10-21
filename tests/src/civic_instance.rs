use std::collections::BTreeMap;

use blake2::{
    digest::{Update, VariableOutput},
    VarBlake2b,
};
use casper_types::{bytesrepr::ToBytes, runtime_args, CLTyped, Key, RuntimeArgs, U256};
use test_env::{Sender, TestContract, TestEnv};

pub type TokenId = String;
pub type Meta = BTreeMap<String, String>;

pub struct CIVICInstance(TestContract);

impl CIVICInstance {
    pub fn new<T: Into<Key>>(
        env: &TestEnv,
        contract_name: &str,
        sender: Sender,
        name: &str,
        symbol: &str,
        meta: Meta,
        admin: T,
    ) -> CIVICInstance {
        CIVICInstance(TestContract::new(
            env,
            "civic-token.wasm",
            contract_name,
            sender,
            runtime_args! {
                "name" => name,
                "symbol" => symbol,
                "meta" => meta,
                "admin" => admin.into()
            },
        ))
    }

    pub fn constructor(&self, sender: Sender, name: &str, symbol: &str, meta: Meta) {
        self.0.call_contract(
            sender,
            "constructor",
            runtime_args! {
            "name" => name,
            "symbol" => symbol,
            "meta" => meta},
        )
    }

    pub fn mint<T: Into<Key>>(
        &self,
        sender: Sender,
        recipient: T,
        token_id: Option<TokenId>,
        token_meta: Meta,
    ) {
        self.0.call_contract(
            sender,
            "mint",
            runtime_args! {
                "recipient" => recipient.into(),
                "token_id" => token_id,
                "token_meta" => token_meta
            },
        )
    }

    pub fn burn<T: Into<Key>>(&self, sender: Sender, owner: T, token_id: TokenId) {
        self.0.call_contract(
            sender,
            "burn",
            runtime_args! {
                "owner" => owner.into(),
                "token_id" => token_id,
            },
        )
    }

    pub fn transfer_from<T: Into<Key>>(
        &self,
        sender: Sender,
        owner: T,
        recipient: T,
        token_ids: Vec<TokenId>,
    ) {
        self.0.call_contract(
            sender,
            "transfer_from",
            runtime_args! {
                "sender" => owner.into(),
                "recipient" => recipient.into(),
                "token_ids" => token_ids
            },
        )
    }

    pub fn set_token_meta(&self, sender: Sender, token_id: TokenId, token_meta: Meta) {
        self.0.call_contract(
            sender,
            "set_token_meta",
            runtime_args! {
                "token_id" => token_id,
                "token_meta" => token_meta
            },
        )
    }

    pub fn update_token_meta(
        &self,
        sender: Sender,
        token_id: TokenId,
        token_meta_key: String,
        token_meta_value: String,
    ) {
        self.0.call_contract(
            sender,
            "update_token_meta",
            runtime_args! {
                "token_id" => token_id,
                "token_meta_key" => token_meta_key,
                "token_meta_value" => token_meta_value
            },
        )
    }

    pub fn grant_gatekeeper<T: Into<Key>>(&self, sender: Sender, gatekeeper: T) {
        self.0.call_contract(
            sender,
            "grant_gatekeeper",
            runtime_args! {
                "gatekeeper" => gatekeeper.into()
            },
        )
    }

    pub fn revoke_gatekeeper<T: Into<Key>>(&self, sender: Sender, gatekeeper: T) {
        self.0.call_contract(
            sender,
            "revoke_gatekeeper",
            runtime_args! {
                "gatekeeper" => gatekeeper.into()
            },
        )
    }

    pub fn grant_admin<T: Into<Key>>(&self, sender: Sender, admin: T) {
        self.0.call_contract(
            sender,
            "grant_admin",
            runtime_args! {
                "admin" => admin.into()
            },
        )
    }

    pub fn revoke_admin<T: Into<Key>>(&self, sender: Sender, admin: T) {
        self.0.call_contract(
            sender,
            "revoke_admin",
            runtime_args! {
                "admin" => admin.into()
            },
        )
    }

    pub fn is_admin<T: Into<Key>>(&self, account: T) -> bool {
        self.0
            .query_dictionary::<()>("admins", key_to_str(&account.into()))
            .is_some()
    }

    pub fn is_gatekeeper<T: Into<Key>>(&self, account: T) -> bool {
        self.0
            .query_dictionary::<()>("gatekeepers", key_to_str(&account.into()))
            .is_some()
    }

    pub fn get_token_by_index<T: Into<Key>>(&self, account: T, index: U256) -> Option<TokenId> {
        self.0.query_dictionary(
            "owned_tokens_by_index",
            key_and_value_to_str(&account.into(), &index),
        )
    }

    pub fn balance_of<T: Into<Key>>(&self, account: T) -> U256 {
        self.0
            .query_dictionary("balances", key_to_str(&account.into()))
            .unwrap_or_default()
    }

    pub fn owner_of(&self, token_id: TokenId) -> Option<Key> {
        self.0.query_dictionary("owners", token_id)
    }

    pub fn token_meta(&self, token_id: TokenId) -> Option<Meta> {
        self.0.query_dictionary("metadata", token_id)
    }

    pub fn name(&self) -> String {
        self.0.query_named_key(String::from("name"))
    }

    pub fn symbol(&self) -> String {
        self.0.query_named_key(String::from("symbol"))
    }

    pub fn total_supply(&self) -> U256 {
        self.0.query_named_key(String::from("total_supply"))
    }

    pub fn meta(&self) -> Meta {
        self.0.query_named_key(String::from("meta"))
    }
}

pub fn key_to_str(key: &Key) -> String {
    match key {
        Key::Account(account) => account.to_string(),
        Key::Hash(package) => hex::encode(package),
        _ => panic!("Unexpected key type"),
    }
}

pub fn key_and_value_to_str<T: CLTyped + ToBytes>(key: &Key, value: &T) -> String {
    let mut hasher = VarBlake2b::new(32).unwrap();
    hasher.update(key.to_bytes().unwrap());
    hasher.update(value.to_bytes().unwrap());
    let mut ret = [0u8; 32];
    hasher.finalize_variable(|hash| ret.clone_from_slice(hash));
    hex::encode(ret)
}
