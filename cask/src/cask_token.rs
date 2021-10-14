#![no_main]
#![no_std]
#[macro_use]
extern crate alloc;

use alloc::{
    boxed::Box,
    collections::{BTreeMap, BTreeSet},
    string::{String, ToString},
    vec::Vec,
};
use cep47::{
    contract_utils::{AdminControl, ContractContext, OnChainContractStorage},
    Error, Meta, TokenId, CEP47,
};
use contract::{
    contract_api::{
        runtime::{self, revert},
        storage,
    },
    unwrap_or_revert::UnwrapOrRevert,
};

use types::{
    contracts::NamedKeys, runtime_args, ApiError, CLType, CLTyped, CLValue, ContractPackageHash,
    EntryPoint, EntryPointAccess, EntryPointType, EntryPoints, Group, Key, Parameter, RuntimeArgs,
    URef, U256,
};

mod minters_control;
use minters_control::MinterControl;

mod custom_data;
use custom_data::Commissions;

pub type Commission = BTreeMap<String, String>;
pub const KYC_HASH: &str = "kyc_package_hash";

#[derive(Default)]
struct CaskToken(OnChainContractStorage);

impl ContractContext<OnChainContractStorage> for CaskToken {
    fn storage(&self) -> &OnChainContractStorage {
        &self.0
    }
}

impl CEP47<OnChainContractStorage> for CaskToken {}
impl AdminControl<OnChainContractStorage> for CaskToken {}
impl MinterControl<OnChainContractStorage> for CaskToken {}

impl CaskToken {
    fn constructor(&mut self, name: String, symbol: String, meta: Meta) {
        CEP47::init(self, name, symbol, meta);
        AdminControl::init(self);
        MinterControl::init(self);
        Commissions::init();
    }

    fn token_commission(&self, token_id: TokenId) -> Option<Commission> {
        Commissions::instance().get(&token_id)
    }

    fn set_token_commission(
        &mut self,
        token_id: TokenId,
        property: String,
        mode: String,
        account: Key,
        value: String,
    ) -> Result<(), Error> {
        if self.owner_of(token_id.clone()).is_none() {
            return Err(Error::TokenIdDoesntExist);
        };
        let commissions_dict = Commissions::instance();
        match mode.as_str() {
            "ADD" => {
                let mut commission = commissions_dict.get(&token_id).unwrap_or_default();
                commission.insert(format!("{}_account", property), account.to_string());
                commission.insert(format!("{}_rate", property), value);
                commissions_dict.set(&token_id, commission);
            }
            "UPDATE" => {
                if account.to_string().is_empty() || value.is_empty() {
                    return Err(Error::WrongArguments);
                }
                let mut commission = commissions_dict.get(&token_id).unwrap_or_default();
                commission.insert(format!("{}_account", property), account.to_string());
                commission.insert(format!("{}_rate", property), value);
                commissions_dict.set(&token_id, commission);
            }
            "DELETE" => {
                let mut commission = commissions_dict.get(&token_id).unwrap_or_default();
                commission.remove(&format!("{}_account", property));
                commission.remove(&format!("{}_rate", property));
                commissions_dict.set(&token_id, commission);
            }
            _ => {
                return Err(Error::WrongArguments);
            }
        }
        Ok(())
    }

    fn mint(
        &mut self,
        recipient: Key,
        token_ids: Option<Vec<TokenId>>,
        token_metas: Vec<Meta>,
        token_commissions: Vec<Commission>,
    ) -> Result<Vec<TokenId>, Error> {
        let caller = CaskToken::default().get_caller();
        if !CaskToken::default().is_minter() && !CaskToken::default().is_admin(caller) {
            revert(ApiError::User(20));
        }
        let mut valid_token_commissions = token_commissions;
        match &token_ids {
            Some(token_ids) => {
                if token_ids.len() != valid_token_commissions.len() {
                    return Err(Error::WrongArguments);
                }
            }
            None => {
                if valid_token_commissions.len() != token_metas.len() {
                    return Err(Error::WrongArguments);
                }
                if valid_token_commissions.is_empty() {
                    valid_token_commissions = vec![Commission::new()];
                }
            }
        }
        let confirmed_token_ids =
            CEP47::mint(self, recipient, token_ids, token_metas).unwrap_or_revert();
        let commissions_dict = Commissions::instance();
        for (token_id, token_commission) in confirmed_token_ids
            .iter()
            .zip(valid_token_commissions.iter())
            .map(|(x, y)| (x, y))
        {
            commissions_dict.set(token_id, token_commission.clone());
        }
        Ok(confirmed_token_ids)
    }

    fn mint_copies(
        &mut self,
        recipient: Key,
        token_ids: Option<Vec<TokenId>>,
        token_meta: Meta,
        token_commission: Commission,
        count: u32,
    ) -> Result<Vec<TokenId>, Error> {
        let caller = CaskToken::default().get_caller();
        if !CaskToken::default().is_minter() && !CaskToken::default().is_admin(caller) {
            revert(ApiError::User(20));
        }
        if let Some(token_ids) = &token_ids {
            if token_ids.len() != count as usize {
                return Err(Error::WrongArguments);
            }
        }
        let token_metas = vec![token_meta; count as usize];
        let token_commissions = vec![token_commission; count as usize];
        self.mint(recipient, token_ids, token_metas, token_commissions)
    }

    fn burn(&mut self, owner: Key, token_ids: Vec<TokenId>) -> Result<(), Error> {
        let caller = CaskToken::default().get_caller();
        if !CaskToken::default().is_minter() && !CaskToken::default().is_admin(caller) {
            revert(ApiError::User(20));
        }

        CEP47::burn_internal(self, owner, token_ids.clone()).unwrap_or_revert();

        let commissions_dict = Commissions::instance();
        for token_id in &token_ids {
            commissions_dict.remove(token_id);
        }
        Ok(())
    }

    fn set_token_meta(&mut self, token_id: TokenId, token_meta: Meta) -> Result<(), Error> {
        let caller = CaskToken::default().get_caller();
        if !CaskToken::default().is_minter() && !CaskToken::default().is_admin(caller) {
            revert(ApiError::User(20));
        }
        CEP47::set_token_meta(self, token_id, token_meta).unwrap_or_revert();
        Ok(())
    }

    fn update_token_meta(
        &mut self,
        token_id: TokenId,
        token_meta_key: String,
        token_meta_value: String,
    ) -> Result<(), Error> {
        let caller = CaskToken::default().get_caller();
        if !CaskToken::default().is_minter() && !CaskToken::default().is_admin(caller) {
            revert(ApiError::User(20));
        }
        let mut token_meta = CaskToken::default()
            .token_meta(token_id.clone())
            .unwrap_or_revert();
        token_meta.insert(token_meta_key, token_meta_value);
        CEP47::set_token_meta(self, token_id, token_meta).unwrap_or_revert();
        Ok(())
    }

    fn update_token_commission(
        &mut self,
        token_id: TokenId,
        property: String,
        mode: String,
        account: Key,
        value: String,
    ) -> Result<(), Error> {
        let caller = CaskToken::default().get_caller();
        if !CaskToken::default().is_admin(caller) {
            revert(ApiError::User(20));
        }
        self.set_token_commission(token_id, property, mode, account, value)
            .unwrap_or_revert();
        Ok(())
    }

    fn get_kyc_hash(&self) -> ContractPackageHash {
        let uref = runtime::get_key(KYC_HASH)
            .unwrap_or_revert_with(ApiError::User(100))
            .into_uref()
            .unwrap_or_revert_with(ApiError::User(101));

        storage::read(uref)
            .unwrap_or_revert_with(ApiError::User(102))
            .unwrap_or_revert_with(ApiError::User(103))
    }

    fn is_kyc_proved(&self, account: Key) -> bool {
        runtime::get_caller();
        runtime::call_versioned_contract::<bool>(
            self.get_kyc_hash(),
            None,
            "is_kyc_proved",
            runtime_args! {"account" => account, "index" => Option::<U256>::None},
        );
        true
    }
}

#[no_mangle]
fn constructor() {
    let name = runtime::get_named_arg::<String>("name");
    let symbol = runtime::get_named_arg::<String>("symbol");
    let meta = runtime::get_named_arg::<Meta>("meta");
    let admin = runtime::get_named_arg::<Key>("admin");
    CaskToken::default().constructor(name, symbol, meta);
    CaskToken::default().add_admin_without_checked(admin);
}

#[no_mangle]
fn name() {
    let ret = CaskToken::default().name();
    runtime::ret(CLValue::from_t(ret).unwrap_or_revert());
}

#[no_mangle]
fn symbol() {
    let ret = CaskToken::default().symbol();
    runtime::ret(CLValue::from_t(ret).unwrap_or_revert());
}

#[no_mangle]
fn meta() {
    let ret = CaskToken::default().meta();
    runtime::ret(CLValue::from_t(ret).unwrap_or_revert());
}

#[no_mangle]
fn total_supply() {
    let ret = CaskToken::default().total_supply();
    runtime::ret(CLValue::from_t(ret).unwrap_or_revert());
}

#[no_mangle]
fn balance_of() {
    let owner = runtime::get_named_arg::<Key>("owner");
    let ret = CaskToken::default().balance_of(owner);
    runtime::ret(CLValue::from_t(ret).unwrap_or_revert());
}

#[no_mangle]
fn owner_of() {
    let token_id = runtime::get_named_arg::<TokenId>("token_id");
    let ret = CaskToken::default().owner_of(token_id);
    runtime::ret(CLValue::from_t(ret).unwrap_or_revert());
}

#[no_mangle]
fn get_token_by_index() {
    let owner = runtime::get_named_arg::<Key>("owner");
    let index = runtime::get_named_arg::<U256>("index");
    let ret = CaskToken::default().get_token_by_index(owner, index);
    runtime::ret(CLValue::from_t(ret).unwrap_or_revert());
}

#[no_mangle]
fn token_meta() {
    let token_id = runtime::get_named_arg::<TokenId>("token_id");
    let ret = CaskToken::default().token_meta(token_id);
    runtime::ret(CLValue::from_t(ret).unwrap_or_revert());
}

#[no_mangle]
fn token_commission() {
    let token_id = runtime::get_named_arg::<TokenId>("token_id");
    let ret = CaskToken::default().token_commission(token_id);
    runtime::ret(CLValue::from_t(ret).unwrap_or_revert());
}

#[no_mangle]
fn set_token_meta() {
    let token_id = runtime::get_named_arg::<TokenId>("token_id");
    let token_meta = runtime::get_named_arg::<Meta>("token_meta");
    CaskToken::default()
        .set_token_meta(token_id, token_meta)
        .unwrap_or_revert();
}

#[no_mangle]
fn update_token_meta() {
    let token_id = runtime::get_named_arg::<TokenId>("token_id");
    let token_meta_key = runtime::get_named_arg::<String>("token_meta_key");
    let token_meta_value = runtime::get_named_arg::<String>("token_meta_value");
    CaskToken::default()
        .update_token_meta(token_id, token_meta_key, token_meta_value)
        .unwrap_or_revert();
}

#[no_mangle]
fn update_token_commission() {
    let token_id = runtime::get_named_arg::<TokenId>("token_id");
    let property = runtime::get_named_arg::<String>("property");
    let account = runtime::get_named_arg::<Key>("account");
    let mode = runtime::get_named_arg::<String>("mode");
    let value = runtime::get_named_arg::<String>("value");
    CaskToken::default()
        .update_token_commission(token_id, property, mode, account, value)
        .unwrap_or_revert();
}

#[no_mangle]
fn mint() {
    let recipient = runtime::get_named_arg::<Key>("recipient");
    let token_ids = runtime::get_named_arg::<Option<Vec<TokenId>>>("token_ids");
    let token_metas = runtime::get_named_arg::<Vec<Meta>>("token_metas");
    let token_commissions = runtime::get_named_arg::<Vec<Commission>>("token_commissions");
    CaskToken::default()
        .mint(recipient, token_ids, token_metas, token_commissions)
        .unwrap_or_revert();
}

#[no_mangle]
fn mint_copies() {
    let recipient = runtime::get_named_arg::<Key>("recipient");
    let token_ids = runtime::get_named_arg::<Option<Vec<TokenId>>>("token_ids");
    let token_meta = runtime::get_named_arg::<Meta>("token_meta");
    let token_commission = runtime::get_named_arg::<Commission>("token_commission");
    let count = runtime::get_named_arg::<u32>("count");
    CaskToken::default()
        .mint_copies(recipient, token_ids, token_meta, token_commission, count)
        .unwrap_or_revert();
}

#[no_mangle]
fn burn() {
    let owner = runtime::get_named_arg::<Key>("owner");
    let token_ids = runtime::get_named_arg::<Vec<TokenId>>("token_ids");
    CaskToken::default()
        .burn(owner, token_ids)
        .unwrap_or_revert()
}

#[no_mangle]
fn transfer() {
    let recipient = runtime::get_named_arg::<Key>("recipient");
    let token_ids = runtime::get_named_arg::<Vec<TokenId>>("token_ids");
    if !CaskToken::default().is_kyc_proved(recipient) {
        revert(ApiError::User(20));
    }
    CaskToken::default()
        .transfer(recipient, token_ids)
        .unwrap_or_revert();
}

#[no_mangle]
fn transfer_from() {
    let sender = runtime::get_named_arg::<Key>("sender");
    let recipient = runtime::get_named_arg::<Key>("recipient");
    let token_ids = runtime::get_named_arg::<Vec<TokenId>>("token_ids");
    let caller = CaskToken::default().get_caller();
    if !CaskToken::default().is_admin(caller) {
        revert(ApiError::User(20));
    }
    CaskToken::default()
        .transfer_from_internal(sender, recipient, token_ids)
        .unwrap_or_revert();
}

#[no_mangle]
fn grant_minter() {
    let minter = runtime::get_named_arg::<Key>("minter");
    CaskToken::default().assert_caller_is_admin();
    CaskToken::default().add_minter(minter);
}

#[no_mangle]
fn revoke_minter() {
    let minter = runtime::get_named_arg::<Key>("minter");
    CaskToken::default().assert_caller_is_admin();
    CaskToken::default().revoke_minter(minter);
}

#[no_mangle]
fn grant_admin() {
    let admin = runtime::get_named_arg::<Key>("admin");
    CaskToken::default().add_admin(admin);
}

#[no_mangle]
fn revoke_admin() {
    let admin = runtime::get_named_arg::<Key>("admin");
    CaskToken::default().disable_admin(admin);
}

#[no_mangle]
fn call() {
    // Read arguments for the constructor call.
    let name: String = runtime::get_named_arg("name");
    let symbol: String = runtime::get_named_arg("symbol");
    let meta: Meta = runtime::get_named_arg("meta");
    let admin: Key = runtime::get_named_arg("admin");
    let kyc_package_hash: [u8; 32] = runtime::get_named_arg::<Key>(KYC_HASH)
        .into_hash()
        .unwrap_or_default();

    let (package_hash, access_token) = storage::create_contract_package_at_hash();
    let mut named_keys = NamedKeys::new();
    let contract_package_hash_wrapped = storage::new_uref(package_hash).into();
    named_keys.insert(
        "contract_package_hash".to_string(),
        contract_package_hash_wrapped,
    );
    named_keys.insert(KYC_HASH.into(), storage::new_uref(kyc_package_hash).into());
    let (contract_hash, _) =
        storage::add_contract_version(package_hash, get_entry_points(), named_keys);

    // Prepare constructor args
    let constructor_args = runtime_args! {
        "name" => name,
        "symbol" => symbol,
        "meta" => meta,
        "admin" => admin
    };

    // Add the constructor group to the package hash with a single URef.
    let constructor_access: URef =
        storage::create_contract_user_group(package_hash, "constructor", 1, Default::default())
            .unwrap_or_revert()
            .pop()
            .unwrap_or_revert();

    // Call the constructor entry point
    let _: () =
        runtime::call_versioned_contract(package_hash, None, "constructor", constructor_args);

    // Remove all URefs from the constructor group, so no one can call it for the second time.
    let mut urefs = BTreeSet::new();
    urefs.insert(constructor_access);
    storage::remove_contract_user_group_urefs(package_hash, "constructor", urefs)
        .unwrap_or_revert();

    // Store contract in the account's named keys.
    let contract_name: alloc::string::String = runtime::get_named_arg("contract_name");
    runtime::put_key(
        &format!("{}_package_hash", contract_name),
        package_hash.into(),
    );
    runtime::put_key(
        &format!("{}_package_hash_wrapped", contract_name),
        contract_package_hash_wrapped,
    );
    runtime::put_key(
        &format!("{}_contract_hash", contract_name),
        contract_hash.into(),
    );
    runtime::put_key(
        &format!("{}_contract_hash_wrapped", contract_name),
        storage::new_uref(contract_hash).into(),
    );
    runtime::put_key(
        &format!("{}_package_access_token", contract_name),
        access_token.into(),
    );
}

fn get_entry_points() -> EntryPoints {
    let mut entry_points = EntryPoints::new();
    entry_points.add_entry_point(EntryPoint::new(
        "constructor",
        vec![
            Parameter::new("name", String::cl_type()),
            Parameter::new("symbol", String::cl_type()),
            Parameter::new("meta", Meta::cl_type()),
            Parameter::new("admin", Key::cl_type()),
        ],
        <()>::cl_type(),
        EntryPointAccess::Groups(vec![Group::new("constructor")]),
        EntryPointType::Contract,
    ));
    entry_points.add_entry_point(EntryPoint::new(
        "name",
        vec![],
        String::cl_type(),
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));
    entry_points.add_entry_point(EntryPoint::new(
        "symbol",
        vec![],
        String::cl_type(),
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));
    entry_points.add_entry_point(EntryPoint::new(
        "meta",
        vec![],
        Meta::cl_type(),
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));
    entry_points.add_entry_point(EntryPoint::new(
        "total_supply",
        vec![],
        U256::cl_type(),
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));
    entry_points.add_entry_point(EntryPoint::new(
        "balance_of",
        vec![Parameter::new("owner", Key::cl_type())],
        U256::cl_type(),
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));
    entry_points.add_entry_point(EntryPoint::new(
        "owner_of",
        vec![Parameter::new("token_id", TokenId::cl_type())],
        CLType::Option(Box::new(CLType::Key)),
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));
    entry_points.add_entry_point(EntryPoint::new(
        "get_token_by_index",
        vec![
            Parameter::new("owner", Key::cl_type()),
            Parameter::new("index", U256::cl_type()),
        ],
        CLType::Option(Box::new(TokenId::cl_type())),
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));
    entry_points.add_entry_point(EntryPoint::new(
        "token_meta",
        vec![Parameter::new("token_id", TokenId::cl_type())],
        Meta::cl_type(),
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));
    entry_points.add_entry_point(EntryPoint::new(
        "token_commission",
        vec![
            Parameter::new("token_id", TokenId::cl_type()),
            Parameter::new("property", String::cl_type()),
        ],
        Commission::cl_type(),
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));
    entry_points.add_entry_point(EntryPoint::new(
        "set_token_meta",
        vec![
            Parameter::new("token_id", TokenId::cl_type()),
            Parameter::new("token_meta", Meta::cl_type()),
        ],
        <()>::cl_type(),
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));
    entry_points.add_entry_point(EntryPoint::new(
        "update_token_meta",
        vec![
            Parameter::new("token_id", TokenId::cl_type()),
            Parameter::new("token_meta_key", String::cl_type()),
            Parameter::new("token_meta_value", String::cl_type()),
        ],
        <()>::cl_type(),
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));
    entry_points.add_entry_point(EntryPoint::new(
        "update_token_commission",
        vec![
            Parameter::new("token_id", TokenId::cl_type()),
            Parameter::new("property", String::cl_type()),
            Parameter::new("account", Key::cl_type()),
            Parameter::new("mode", String::cl_type()),
            Parameter::new("value", String::cl_type()),
        ],
        <()>::cl_type(),
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));
    entry_points.add_entry_point(EntryPoint::new(
        "mint",
        vec![
            Parameter::new("recipient", Key::cl_type()),
            Parameter::new(
                "token_ids",
                CLType::Option(Box::new(CLType::List(Box::new(TokenId::cl_type())))),
            ),
            Parameter::new("token_metas", CLType::List(Box::new(Meta::cl_type()))),
            Parameter::new(
                "token_commissions",
                CLType::List(Box::new(Commission::cl_type())),
            ),
        ],
        <()>::cl_type(),
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));
    entry_points.add_entry_point(EntryPoint::new(
        "mint_copies",
        vec![
            Parameter::new("recipient", Key::cl_type()),
            Parameter::new(
                "token_ids",
                CLType::Option(Box::new(CLType::List(Box::new(TokenId::cl_type())))),
            ),
            Parameter::new("token_meta", Meta::cl_type()),
            Parameter::new("count", CLType::U32),
        ],
        <()>::cl_type(),
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));
    entry_points.add_entry_point(EntryPoint::new(
        "burn",
        vec![
            Parameter::new("owner", Key::cl_type()),
            Parameter::new("token_ids", CLType::List(Box::new(TokenId::cl_type()))),
        ],
        <()>::cl_type(),
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));
    entry_points.add_entry_point(EntryPoint::new(
        "transfer",
        vec![
            Parameter::new("recipient", Key::cl_type()),
            Parameter::new("token_ids", CLType::List(Box::new(TokenId::cl_type()))),
        ],
        <()>::cl_type(),
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));
    entry_points.add_entry_point(EntryPoint::new(
        "transfer_from",
        vec![
            Parameter::new("sender", Key::cl_type()),
            Parameter::new("recipient", Key::cl_type()),
            Parameter::new("token_ids", CLType::List(Box::new(TokenId::cl_type()))),
        ],
        <()>::cl_type(),
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));
    entry_points.add_entry_point(EntryPoint::new(
        "grant_minter",
        vec![Parameter::new("minter", Key::cl_type())],
        <()>::cl_type(),
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));
    entry_points.add_entry_point(EntryPoint::new(
        "revoke_minter",
        vec![Parameter::new("minter", Key::cl_type())],
        <()>::cl_type(),
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));
    entry_points.add_entry_point(EntryPoint::new(
        "grant_admin",
        vec![Parameter::new("admin", Key::cl_type())],
        <()>::cl_type(),
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));
    entry_points.add_entry_point(EntryPoint::new(
        "revoke_admin",
        vec![Parameter::new("admin", Key::cl_type())],
        <()>::cl_type(),
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));
    entry_points
}
