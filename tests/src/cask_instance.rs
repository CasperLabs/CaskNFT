#![allow(non_camel_case_types)]
use std::collections::BTreeMap;

use blake2::{
    digest::{Update, VariableOutput},
    VarBlake2b,
};
use casper_types::{bytesrepr::ToBytes, runtime_args, CLTyped, Key, RuntimeArgs, U256};
use test_env::{Sender, TestContract, TestEnv};

pub type TokenId = String;
pub type Meta = BTreeMap<String, String>;
pub type Gauge = BTreeMap<String, String>;
pub type Warehouse = BTreeMap<String, String>;
pub type Commission = BTreeMap<String, SubCommission>;
pub type SubCommission = BTreeMap<String, String>;

pub struct METADATA(Meta, Gauge, Warehouse, Commission);
impl METADATA {
    pub fn new(meta: Meta, gauge: Gauge, warehouse: Warehouse, commission: Commission) -> METADATA {
        METADATA(meta, gauge, warehouse, commission)
    }
}

pub struct METADATA_VEC(Vec<Meta>, Vec<Gauge>, Vec<Warehouse>, Vec<Commission>);
impl METADATA_VEC {
    pub fn new(
        metas: Vec<Meta>,
        gauges: Vec<Gauge>,
        warehouses: Vec<Warehouse>,
        commissions: Vec<Commission>,
    ) -> METADATA_VEC {
        METADATA_VEC(metas, gauges, warehouses, commissions)
    }
}

pub struct CaskInstance(TestContract);

impl CaskInstance {
    pub fn new<T: Into<Key>>(
        env: &TestEnv,
        contract_name: &str,
        sender: Sender,
        name: &str,
        symbol: &str,
        meta: Meta,
        admin: T,
    ) -> CaskInstance {
        CaskInstance(TestContract::new(
            env,
            "cask-token.wasm",
            contract_name,
            sender,
            runtime_args! {
                "name" => name,
                "symbol" => symbol,
                "meta" => meta,
                "admin" => admin.into(),
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
        );
    }

    pub fn grant_admin<T: Into<Key>>(&self, sender: Sender, admin: T) {
        self.0.call_contract(
            sender,
            "grant_admin",
            runtime_args! {
            "admin" => admin.into()},
        );
    }

    pub fn revoke_admin<T: Into<Key>>(&self, sender: Sender, admin: T) {
        self.0.call_contract(
            sender,
            "revoke_admin",
            runtime_args! {
            "admin" => admin.into()},
        );
    }

    pub fn grant_minter<T: Into<Key>>(&self, sender: Sender, minter: T) {
        self.0.call_contract(
            sender,
            "grant_minter",
            runtime_args! {
            "minter" => minter.into()},
        );
    }

    pub fn revoke_minter<T: Into<Key>>(&self, sender: Sender, minter: T) {
        self.0.call_contract(
            sender,
            "revoke_minter",
            runtime_args! {
            "minter" => minter.into()},
        );
    }

    pub fn mint<T: Into<Key>>(
        &self,
        sender: Sender,
        recipient: T,
        token_ids: Option<Vec<TokenId>>,
        token_metadatas: METADATA_VEC,
    ) {
        self.0.call_contract(
            sender,
            "mint",
            runtime_args! {
                "recipient" => recipient.into(),
                "token_ids" => token_ids,
                "token_metas" => token_metadatas.0,
                "token_gauges" => token_metadatas.1,
                "token_warehouses" => token_metadatas.2,
                "token_commissions" => token_metadatas.3
            },
        )
    }

    pub fn mint_copies<T: Into<Key>>(
        &self,
        sender: Sender,
        recipient: T,
        token_ids: Option<Vec<TokenId>>,
        token_metadata: METADATA,
        count: u32,
    ) {
        self.0.call_contract(
            sender,
            "mint_copies",
            runtime_args! {
                "recipient" => recipient.into(),
                "token_ids" => token_ids,
                "token_meta" => token_metadata.0,
                "token_gauge" => token_metadata.1,
                "token_warehouse" => token_metadata.2,
                "token_commission" => token_metadata.3,
                "count" => count
            },
        )
    }

    pub fn transfer<T: Into<Key>>(&self, sender: Sender, recipient: T, token_ids: Vec<TokenId>) {
        self.0.call_contract(
            sender,
            "transfer",
            runtime_args! {
                "recipient" => recipient.into(),
                "token_ids" => token_ids
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

    pub fn update_token_meta(&self, sender: Sender, token_id: TokenId, token_meta: Meta) {
        self.0.call_contract(
            sender,
            "update_token_meta",
            runtime_args! {
                "token_id" => token_id,
                "token_meta" => token_meta
            },
        )
    }

    pub fn update_token_gauge(&self, sender: Sender, token_id: TokenId, token_gauge: Gauge) {
        self.0.call_contract(
            sender,
            "update_token_gauge",
            runtime_args! {
                "token_id" => token_id,
                "token_gauge" => token_gauge
            },
        )
    }

    pub fn update_token_warehouse(
        &self,
        sender: Sender,
        token_id: TokenId,
        token_warehouse: Warehouse,
    ) {
        self.0.call_contract(
            sender,
            "update_token_warehouse",
            runtime_args! {
                "token_id" => token_id,
                "token_warehouse" => token_warehouse
            },
        )
    }

    pub fn update_token_commission<T: Into<Key>>(
        &self,
        sender: Sender,
        token_id: TokenId,
        property: String,
        account: T,
        mode: String,
        value: String,
    ) {
        self.0.call_contract(
            sender,
            "update_token_commission",
            runtime_args! {
                "token_id" => token_id,
                "property" => property,
                "account" => account.into(),
                "mode" => mode,
                "value" => value
            },
        )
    }

    pub fn burn<T: Into<Key>>(&self, sender: Sender, owner: T, token_ids: Vec<TokenId>) {
        self.0.call_contract(
            sender,
            "burn",
            runtime_args! {
                "owner" => owner.into(),
                "token_ids" => token_ids
            },
        )
    }

    pub fn is_admin<T: Into<Key>>(&self, account: T) -> bool {
        self.0
            .query_dictionary::<()>("admins", key_to_str(&account.into()))
            .is_some()
    }

    pub fn is_minter<T: Into<Key>>(&self, account: T) -> bool {
        self.0
            .query_dictionary::<()>("minters", key_to_str(&account.into()))
            .is_some()
    }

    pub fn token_meta(&self, token_id: TokenId) -> Option<Meta> {
        self.0.query_dictionary("metadata", token_id)
    }

    pub fn token_gauge(&self, token_id: TokenId) -> Option<Gauge> {
        self.0.query_dictionary("gauges", token_id)
    }

    pub fn token_warehouse(&self, token_id: TokenId) -> Option<Warehouse> {
        self.0.query_dictionary("warehouses", token_id)
    }

    pub fn token_commission_by_property(
        &self,
        token_id: TokenId,
        property: String,
    ) -> Option<SubCommission> {
        let commission: Commission = self
            .0
            .query_dictionary("commissions", token_id)
            .unwrap_or_default();
        if let Some(t) = commission.get(&property) {
            return Some(t.clone());
        }
        None
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
