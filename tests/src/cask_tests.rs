use casper_engine_test_support::AccountHash;
use casper_types::{Key, U256};
use test_env::{Sender, TestEnv};

use crate::cask_instance::{
    CaskInstance, Commission, Gauge, Meta, SubCommission, TokenId, Warehouse,
};

const NAME: &str = "CaskNFT";
const SYMBOL: &str = "CNFT";

mod meta {
    use super::Meta;
    pub fn contract_meta() -> Meta {
        let mut meta = Meta::new();
        meta.insert("origin".to_string(), "small".to_string());
        meta
    }

    pub fn big_cask() -> Meta {
        let mut meta = Meta::new();
        meta.insert("size".to_string(), "big".to_string());
        meta
    }

    pub fn medium_cask() -> Meta {
        let mut meta = Meta::new();
        meta.insert("size".to_string(), "medium".to_string());
        meta
    }
}

mod gauge {
    use super::Gauge;
    pub fn alchol_gauge() -> Gauge {
        let mut gauge = Gauge::new();
        gauge.insert("alchol_percentage".to_string(), "40".to_string());
        gauge
    }

    pub fn phenol_gauge() -> Gauge {
        let mut gauge = Gauge::new();
        gauge.insert("phenol".to_string(), "yes".to_string());
        gauge
    }
}

mod warehouse {
    use super::Warehouse;
    pub fn west_warehouse() -> Warehouse {
        let mut warehouse = Warehouse::new();
        warehouse.insert("location".to_string(), "west".to_string());
        warehouse
    }

    pub fn south_warehouse() -> Warehouse {
        let mut warehouse = Warehouse::new();
        warehouse.insert("location".to_string(), "south".to_string());
        warehouse
    }
}

mod commission {
    use casper_types::Key;

    use super::{Commission, SubCommission};
    pub fn commission(
        properties: Vec<String>,
        accounts: Vec<Key>,
        rates: Vec<String>,
    ) -> Commission {
        let mut commission = Commission::new();
        for (property, account, rate) in properties
            .iter()
            .zip(accounts.iter())
            .zip(rates.iter())
            .map(|((x, y), z)| (x, y, z))
        {
            let sub = sub_commission(*account, rate.clone());
            commission.insert(property.clone(), sub);
        }
        commission
    }

    pub fn sub_commission(account: Key, rate: String) -> SubCommission {
        let mut sub_commission = SubCommission::new();
        sub_commission.insert("Rate".to_string(), rate);
        sub_commission.insert("Account".to_string(), account.to_string());
        sub_commission
    }
}

fn deploy() -> (TestEnv, CaskInstance, AccountHash) {
    let env = TestEnv::new();
    let owner = env.next_user();
    let token = CaskInstance::new(
        &env,
        NAME,
        Sender(owner),
        NAME,
        SYMBOL,
        meta::contract_meta(),
        owner,
    );
    (env, token, owner)
}

#[test]
fn test_deploy() {
    let (_, token, owner) = deploy();
    assert_eq!(token.name(), NAME);
    assert_eq!(token.symbol(), SYMBOL);
    assert_eq!(token.meta(), meta::contract_meta());
    assert_eq!(token.total_supply(), U256::zero());
    assert!(token.is_admin(owner));
}

#[test]
fn test_grant_admin() {
    let (env, token, owner) = deploy();
    let user = env.next_user();

    token.grant_admin(Sender(owner), user);
    assert!(token.is_admin(user));
}

#[test]
fn test_revoke_admin() {
    let (env, token, owner) = deploy();
    let user = env.next_user();

    token.grant_admin(Sender(owner), user);
    assert!(token.is_admin(user));

    token.revoke_admin(Sender(owner), user);
    assert!(!token.is_admin(user));
}

#[test]
fn test_grant_minter() {
    let (env, token, owner) = deploy();
    let alice = env.next_user();
    let bob = env.next_user();

    token.grant_admin(Sender(owner), alice);
    token.grant_minter(Sender(alice), bob);
    assert!(token.is_minter(bob));
}

#[test]
fn test_revoke_minter() {
    let (env, token, owner) = deploy();
    let alice = env.next_user();
    let bob = env.next_user();

    token.grant_minter(Sender(owner), bob);
    assert!(token.is_minter(bob));

    token.grant_admin(Sender(owner), alice);
    token.revoke_minter(Sender(alice), bob);
    assert!(!token.is_minter(bob));
}

#[test]
fn test_mint_from_minter() {
    let (env, token, owner) = deploy();
    let ali = env.next_user();
    let bob = env.next_user();
    let token_id = TokenId::from("custom_token_id");
    let token_meta = meta::big_cask();
    let token_gauge = gauge::alchol_gauge();
    let token_warehouse = warehouse::west_warehouse();
    let token_commission = commission::commission(
        vec!["artist".to_string(), "broker".to_string()],
        vec![ali.into(), bob.into()],
        vec!["10".to_string(), "12".to_string()],
    );

    token.grant_minter(Sender(owner), ali);

    token.mint(
        Sender(ali),
        bob,
        Some(vec![token_id.clone()]),
        vec![token_meta.clone()],
        vec![token_gauge.clone()],
        vec![token_warehouse.clone()],
        vec![token_commission],
    );

    let user_token_meta = token.token_meta(token_id.clone());
    assert_eq!(user_token_meta.unwrap(), token_meta);

    let user_token_gauge = token.token_gauge(token_id.clone());
    assert_eq!(user_token_gauge.unwrap(), token_gauge);

    let user_token_warehouse = token.token_warehouse(token_id.clone());
    assert_eq!(user_token_warehouse.unwrap(), token_warehouse);

    let user_token_commission_artist =
        token.token_commission_by_property(token_id.clone(), String::from("artist"));
    assert_eq!(
        user_token_commission_artist.unwrap(),
        commission::sub_commission(ali.into(), "10".to_string())
    );

    let first_user_token = token.get_token_by_index(Key::Account(bob), U256::zero());
    assert_eq!(first_user_token, Some(token_id));
}

#[test]
#[should_panic]
fn test_mint_with_wrong_arguments() {
    let (env, token, owner) = deploy();
    let ali = env.next_user();
    let bob = env.next_user();
    let token_meta = meta::big_cask();
    let token_commission = commission::commission(
        vec!["artist".to_string(), "broker".to_string()],
        vec![ali.into(), bob.into()],
        vec!["10".to_string(), "12".to_string()],
    );

    token.grant_minter(Sender(owner), ali);

    token.mint(
        Sender(ali),
        bob,
        None,
        vec![token_meta],
        vec![],
        vec![],
        vec![token_commission],
    );
}

#[test]
#[should_panic]
fn test_mint_from_non_minter() {
    let (env, token, _) = deploy();
    let ali = env.next_user();
    let bob = env.next_user();
    let token_id = TokenId::from("custom_token_id");
    let token_meta = meta::big_cask();
    let token_gauge = gauge::alchol_gauge();
    let token_warehouse = warehouse::west_warehouse();
    let token_commission = commission::commission(
        vec!["artist".to_string(), "broker".to_string()],
        vec![ali.into(), bob.into()],
        vec!["10".to_string(), "12".to_string()],
    );

    token.mint(
        Sender(ali),
        bob,
        Some(vec![token_id]),
        vec![token_meta],
        vec![token_gauge],
        vec![token_warehouse],
        vec![token_commission],
    );
}

#[test]
fn test_mint_copies_from_minter() {
    let (env, token, owner) = deploy();
    let ali = env.next_user();
    let bob = env.next_user();
    let token_meta = meta::big_cask();
    let token_gauge = gauge::alchol_gauge();
    let token_warehouse = warehouse::west_warehouse();
    let token_commission = commission::commission(
        vec!["artist".to_string(), "broker".to_string()],
        vec![ali.into(), bob.into()],
        vec!["10".to_string(), "12".to_string()],
    );

    token.grant_minter(Sender(owner), ali);

    token.mint_copies(
        Sender(ali),
        bob,
        None,
        token_meta.clone(),
        token_gauge,
        token_warehouse,
        token_commission,
        3,
    );

    let first_user_token = token.get_token_by_index(Key::Account(bob), U256::from(0));
    let second_user_token = token.get_token_by_index(Key::Account(bob), U256::from(1));
    let third_user_token = token.get_token_by_index(Key::Account(bob), U256::from(2));
    let fourth_user_token = token.get_token_by_index(Key::Account(bob), U256::from(3));
    assert_eq!(token.total_supply(), U256::from(3));
    assert_eq!(token.balance_of(Key::Account(bob)), U256::from(3));
    assert_eq!(fourth_user_token, None);
    assert_eq!(
        token.owner_of(first_user_token.clone().unwrap()).unwrap(),
        Key::Account(bob)
    );
    assert_eq!(
        token.owner_of(second_user_token.clone().unwrap()).unwrap(),
        Key::Account(bob)
    );
    assert_eq!(
        token.owner_of(third_user_token.unwrap()).unwrap(),
        Key::Account(bob)
    );

    let mut user_token_meta = token.token_meta(first_user_token.unwrap());
    assert_eq!(user_token_meta.unwrap(), token_meta);

    user_token_meta = token.token_meta(second_user_token.unwrap());
    assert_eq!(user_token_meta.unwrap(), token_meta);
}

#[test]
fn test_burn_from_minter() {
    let (env, token, owner) = deploy();
    let ali = env.next_user();
    let bob = env.next_user();
    let token_meta = meta::big_cask();
    let token_gauge = gauge::alchol_gauge();
    let token_warehouse = warehouse::west_warehouse();
    let token_commission = commission::commission(
        vec!["artist".to_string(), "broker".to_string()],
        vec![ali.into(), bob.into()],
        vec!["10".to_string(), "12".to_string()],
    );

    token.mint_copies(
        Sender(owner),
        bob,
        None,
        token_meta,
        token_gauge,
        token_warehouse,
        token_commission,
        2,
    );

    token.grant_minter(Sender(owner), ali);

    let first_user_token = token.get_token_by_index(Key::Account(bob), U256::from(0));
    let second_user_token = token.get_token_by_index(Key::Account(bob), U256::from(1));
    token.burn(Sender(ali), bob, vec![first_user_token.unwrap()]);
    assert_eq!(token.total_supply(), U256::from(1));
    assert_eq!(token.balance_of(Key::Account(bob)), U256::from(1));

    let new_first_user_token = token.get_token_by_index(Key::Account(bob), U256::from(0));
    let new_second_user_token = token.get_token_by_index(Key::Account(bob), U256::from(1));
    assert_eq!(new_first_user_token, second_user_token);
    assert_eq!(new_second_user_token, None);
}

#[test]
#[should_panic]
fn test_burn_from_non_minter() {
    let (env, token, owner) = deploy();
    let ali = env.next_user();
    let bob = env.next_user();
    let token_meta = meta::big_cask();
    let token_gauge = gauge::alchol_gauge();
    let token_warehouse = warehouse::west_warehouse();
    let token_commission = commission::commission(
        vec!["artist".to_string(), "broker".to_string()],
        vec![ali.into(), bob.into()],
        vec!["10".to_string(), "12".to_string()],
    );

    token.mint_copies(
        Sender(owner),
        bob,
        None,
        token_meta,
        token_gauge,
        token_warehouse,
        token_commission,
        2,
    );

    let first_user_token = token.get_token_by_index(Key::Account(bob), U256::from(0));
    token.burn(Sender(ali), bob, vec![first_user_token.unwrap()]);
}

#[test]
fn test_transfer_from_owner() {
    let (env, token, owner) = deploy();
    let ali = env.next_user();
    let bob = env.next_user();
    let token_meta = meta::big_cask();
    let token_gauge = gauge::alchol_gauge();
    let token_warehouse = warehouse::west_warehouse();
    let token_commission = commission::commission(
        vec!["artist".to_string(), "broker".to_string()],
        vec![ali.into(), bob.into()],
        vec!["10".to_string(), "12".to_string()],
    );

    token.mint_copies(
        Sender(owner),
        ali,
        None,
        token_meta,
        token_gauge,
        token_warehouse,
        token_commission,
        2,
    );
    let first_ali_token = token.get_token_by_index(Key::Account(ali), U256::from(0));
    let second_ali_token = token.get_token_by_index(Key::Account(ali), U256::from(1));

    assert_eq!(token.total_supply(), U256::from(2));
    assert_eq!(token.balance_of(Key::Account(ali)), U256::from(2));
    assert_eq!(
        token.owner_of(first_ali_token.clone().unwrap()).unwrap(),
        Key::Account(ali)
    );
    assert_eq!(
        token.owner_of(second_ali_token.unwrap()).unwrap(),
        Key::Account(ali)
    );
    token.transfer_from(Sender(ali), ali, bob, vec![first_ali_token.unwrap()]);
    let new_first_ali_token = token.get_token_by_index(Key::Account(ali), U256::from(0));
    let new_second_ali_token = token.get_token_by_index(Key::Account(ali), U256::from(1));
    let new_first_bob_token = token.get_token_by_index(Key::Account(bob), U256::from(0));
    let new_second_bob_token = token.get_token_by_index(Key::Account(bob), U256::from(1));
    println!("{:?}", new_first_ali_token);
    println!("{:?}", new_second_ali_token);
    println!("{:?}", new_first_bob_token);
    println!("{:?}", new_second_bob_token);
    assert_eq!(token.total_supply(), U256::from(2));
    assert_eq!(token.balance_of(Key::Account(ali)), U256::from(1));
    assert_eq!(token.balance_of(Key::Account(bob)), U256::from(1));
    assert_eq!(
        token.owner_of(new_first_ali_token.unwrap()).unwrap(),
        Key::Account(ali)
    );
    assert_eq!(
        token.owner_of(new_first_bob_token.unwrap()).unwrap(),
        Key::Account(bob)
    );
    assert_eq!(new_second_ali_token, None);
    assert_eq!(new_second_bob_token, None);
}

#[test]
fn test_transfer_from_admin() {
    let (env, token, owner) = deploy();
    let ali = env.next_user();
    let bob = env.next_user();
    let token_meta = meta::big_cask();
    let token_gauge = gauge::alchol_gauge();
    let token_warehouse = warehouse::west_warehouse();
    let token_commission = commission::commission(
        vec!["artist".to_string(), "broker".to_string()],
        vec![ali.into(), bob.into()],
        vec!["10".to_string(), "12".to_string()],
    );

    token.mint_copies(
        Sender(owner),
        ali,
        None,
        token_meta,
        token_gauge,
        token_warehouse,
        token_commission,
        2,
    );
    let first_ali_token = token.get_token_by_index(Key::Account(ali), U256::from(0));
    let second_ali_token = token.get_token_by_index(Key::Account(ali), U256::from(1));

    assert_eq!(token.total_supply(), U256::from(2));
    assert_eq!(token.balance_of(Key::Account(ali)), U256::from(2));
    assert_eq!(
        token.owner_of(first_ali_token.clone().unwrap()).unwrap(),
        Key::Account(ali)
    );
    assert_eq!(
        token.owner_of(second_ali_token.unwrap()).unwrap(),
        Key::Account(ali)
    );
    token.transfer_from(Sender(owner), ali, bob, vec![first_ali_token.unwrap()]);
    let new_first_ali_token = token.get_token_by_index(Key::Account(ali), U256::from(0));
    let new_second_ali_token = token.get_token_by_index(Key::Account(ali), U256::from(1));
    let new_first_bob_token = token.get_token_by_index(Key::Account(bob), U256::from(0));
    let new_second_bob_token = token.get_token_by_index(Key::Account(bob), U256::from(1));
    println!("{:?}", new_first_ali_token);
    println!("{:?}", new_second_ali_token);
    println!("{:?}", new_first_bob_token);
    println!("{:?}", new_second_bob_token);
    assert_eq!(token.total_supply(), U256::from(2));
    assert_eq!(token.balance_of(Key::Account(ali)), U256::from(1));
    assert_eq!(token.balance_of(Key::Account(bob)), U256::from(1));
    assert_eq!(
        token.owner_of(new_first_ali_token.unwrap()).unwrap(),
        Key::Account(ali)
    );
    assert_eq!(
        token.owner_of(new_first_bob_token.unwrap()).unwrap(),
        Key::Account(bob)
    );
    assert_eq!(new_second_ali_token, None);
    assert_eq!(new_second_bob_token, None);
}

#[test]
#[should_panic]
fn test_transfer_from_minter() {
    let (env, token, owner) = deploy();
    let ali = env.next_user();
    let bob = env.next_user();
    let token_meta = meta::big_cask();
    let token_gauge = gauge::alchol_gauge();
    let token_warehouse = warehouse::west_warehouse();
    let token_commission = commission::commission(
        vec!["artist".to_string(), "broker".to_string()],
        vec![ali.into(), bob.into()],
        vec!["10".to_string(), "12".to_string()],
    );

    token.mint_copies(
        Sender(owner),
        ali,
        None,
        token_meta,
        token_gauge,
        token_warehouse,
        token_commission,
        2,
    );
    let first_ali_token = token.get_token_by_index(Key::Account(ali), U256::from(0));
    let second_ali_token = token.get_token_by_index(Key::Account(ali), U256::from(1));

    assert_eq!(token.total_supply(), U256::from(2));
    assert_eq!(token.balance_of(Key::Account(ali)), U256::from(2));
    assert_eq!(
        token.owner_of(first_ali_token.clone().unwrap()).unwrap(),
        Key::Account(ali)
    );
    assert_eq!(
        token.owner_of(second_ali_token.unwrap()).unwrap(),
        Key::Account(ali)
    );
    token.grant_minter(Sender(owner), bob);
    token.transfer_from(Sender(bob), ali, bob, vec![first_ali_token.unwrap()]);
}

#[test]
fn test_transfer() {
    let (env, token, owner) = deploy();
    let ali = env.next_user();
    let bob = env.next_user();
    let token_meta = meta::big_cask();
    let token_gauge = gauge::alchol_gauge();
    let token_warehouse = warehouse::west_warehouse();
    let token_commission = commission::commission(
        vec!["artist".to_string(), "broker".to_string()],
        vec![ali.into(), bob.into()],
        vec!["10".to_string(), "12".to_string()],
    );

    token.mint_copies(
        Sender(owner),
        ali,
        None,
        token_meta,
        token_gauge,
        token_warehouse,
        token_commission,
        2,
    );
    let first_ali_token = token.get_token_by_index(Key::Account(ali), U256::from(0));
    let second_ali_token = token.get_token_by_index(Key::Account(ali), U256::from(1));

    assert_eq!(token.total_supply(), U256::from(2));
    assert_eq!(token.balance_of(Key::Account(ali)), U256::from(2));
    assert_eq!(
        token.owner_of(first_ali_token.clone().unwrap()).unwrap(),
        Key::Account(ali)
    );
    assert_eq!(
        token.owner_of(second_ali_token.unwrap()).unwrap(),
        Key::Account(ali)
    );
    token.transfer(Sender(ali), bob, vec![first_ali_token.unwrap()]);
    let new_first_ali_token = token.get_token_by_index(Key::Account(ali), U256::from(0));
    let new_second_ali_token = token.get_token_by_index(Key::Account(ali), U256::from(1));
    let new_first_bob_token = token.get_token_by_index(Key::Account(bob), U256::from(0));
    let new_second_bob_token = token.get_token_by_index(Key::Account(bob), U256::from(1));
    println!("{:?}", new_first_ali_token);
    println!("{:?}", new_second_ali_token);
    println!("{:?}", new_first_bob_token);
    println!("{:?}", new_second_bob_token);
    assert_eq!(token.total_supply(), U256::from(2));
    assert_eq!(token.balance_of(Key::Account(ali)), U256::from(1));
    assert_eq!(token.balance_of(Key::Account(bob)), U256::from(1));
    assert_eq!(
        token.owner_of(new_first_ali_token.unwrap()).unwrap(),
        Key::Account(ali)
    );
    assert_eq!(
        token.owner_of(new_first_bob_token.unwrap()).unwrap(),
        Key::Account(bob)
    );
    assert_eq!(new_second_ali_token, None);
    assert_eq!(new_second_bob_token, None);
}

#[test]
fn test_token_meta() {
    let (env, token, owner) = deploy();
    let ali = env.next_user();
    let bob = env.next_user();
    let token_id = TokenId::from("123456");
    let token_meta = meta::big_cask();
    let token_gauge = gauge::alchol_gauge();
    let token_warehouse = warehouse::west_warehouse();
    let token_commission = commission::commission(
        vec!["artist".to_string(), "broker".to_string()],
        vec![ali.into(), bob.into()],
        vec!["10".to_string(), "12".to_string()],
    );

    token.mint_copies(
        Sender(owner),
        ali,
        Some(vec![token_id.clone()]),
        token_meta.clone(),
        token_gauge.clone(),
        token_warehouse.clone(),
        token_commission,
        1,
    );

    let user_token_meta = token.token_meta(token_id.clone());
    assert_eq!(user_token_meta.unwrap(), token_meta);

    let user_token_gauge = token.token_gauge(token_id.clone());
    assert_eq!(user_token_gauge.unwrap(), token_gauge);

    let user_token_warehouse = token.token_warehouse(token_id.clone());
    assert_eq!(user_token_warehouse.unwrap(), token_warehouse);

    let user_token_commission_artist =
        token.token_commission_by_property(token_id.clone(), String::from("broker"));
    assert_eq!(
        user_token_commission_artist.unwrap(),
        commission::sub_commission(bob.into(), "12".to_string())
    );

    let first_user_token = token.get_token_by_index(Key::Account(ali), U256::zero());
    assert_eq!(first_user_token, Some(token_id));
}

#[test]
fn test_token_metadata_update_from_minter() {
    let (env, token, owner) = deploy();
    let ali = env.next_user();
    let bob = env.next_user();
    let token_id = TokenId::from("123456");
    let token_meta = meta::big_cask();
    let token_gauge = gauge::alchol_gauge();
    let token_warehouse = warehouse::west_warehouse();
    let token_commission = commission::commission(
        vec!["artist".to_string(), "broker".to_string()],
        vec![ali.into(), bob.into()],
        vec!["10".to_string(), "12".to_string()],
    );

    token.mint_copies(
        Sender(owner),
        ali,
        Some(vec![token_id.clone()]),
        token_meta,
        token_gauge,
        token_warehouse,
        token_commission,
        1,
    );
    token.grant_minter(Sender(owner), ali);
    token.update_token_meta(Sender(ali), token_id.clone(), meta::medium_cask());
    token.update_token_gauge(Sender(ali), token_id.clone(), gauge::phenol_gauge());
    token.update_token_warehouse(Sender(ali), token_id.clone(), warehouse::south_warehouse());
    assert_eq!(token.token_meta(token_id).unwrap(), meta::medium_cask());
}

#[test]
#[should_panic]
fn test_token_metadata_update_from_owner() {
    let (env, token, owner) = deploy();
    let ali = env.next_user();
    let bob = env.next_user();
    let token_id = TokenId::from("123456");
    let token_meta = meta::big_cask();
    let token_gauge = gauge::alchol_gauge();
    let token_warehouse = warehouse::west_warehouse();
    let token_commission = commission::commission(
        vec!["artist".to_string(), "broker".to_string()],
        vec![ali.into(), bob.into()],
        vec!["10".to_string(), "12".to_string()],
    );

    token.mint_copies(
        Sender(owner),
        ali,
        Some(vec![token_id.clone()]),
        token_meta,
        token_gauge,
        token_warehouse,
        token_commission,
        1,
    );
    token.update_token_meta(Sender(ali), token_id, meta::medium_cask());
}

#[test]
fn test_token_commission_update_from_admin() {
    let (env, token, owner) = deploy();
    let ali = env.next_user();
    let bob = env.next_user();
    let token_id = TokenId::from("123456");
    let token_meta = meta::big_cask();
    let token_gauge = gauge::alchol_gauge();
    let token_warehouse = warehouse::west_warehouse();
    let token_commission = commission::commission(
        vec!["artist".to_string()],
        vec![ali.into()],
        vec!["10".to_string()],
    );

    token.mint_copies(
        Sender(owner),
        ali,
        Some(vec![token_id.clone()]),
        token_meta,
        token_gauge,
        token_warehouse,
        token_commission,
        1,
    );
    token.update_token_commission(
        Sender(owner),
        token_id.clone(),
        "artist".to_string(),
        owner,
        "UPDATE".to_string(),
        "12".to_string(),
    );
    let mut user_token_sub_commission =
        token.token_commission_by_property(token_id.clone(), String::from("artist"));
    assert_eq!(
        user_token_sub_commission.unwrap(),
        commission::sub_commission(owner.into(), "12".to_string())
    );
    token.update_token_commission(
        Sender(owner),
        token_id.clone(),
        "broker".to_string(),
        bob,
        "ADD".to_string(),
        "10".to_string(),
    );
    user_token_sub_commission =
        token.token_commission_by_property(token_id.clone(), String::from("broker"));
    assert_eq!(
        user_token_sub_commission.unwrap(),
        commission::sub_commission(bob.into(), "10".to_string())
    );
    token.update_token_commission(
        Sender(owner),
        token_id.clone(),
        "broker".to_string(),
        bob,
        "DELETE".to_string(),
        String::new(),
    );
    user_token_sub_commission =
        token.token_commission_by_property(token_id, String::from("broker"));
    assert_eq!(user_token_sub_commission, None);
}

#[test]
#[should_panic]
fn test_token_commission_update_from_minter() {
    let (env, token, owner) = deploy();
    let ali = env.next_user();
    let bob = env.next_user();
    let token_id = TokenId::from("123456");
    let token_meta = meta::big_cask();
    let token_gauge = gauge::alchol_gauge();
    let token_warehouse = warehouse::west_warehouse();
    let token_commission = commission::commission(
        vec!["artist".to_string()],
        vec![ali.into()],
        vec!["10".to_string()],
    );

    token.mint_copies(
        Sender(owner),
        ali,
        Some(vec![token_id.clone()]),
        token_meta,
        token_gauge,
        token_warehouse,
        token_commission,
        1,
    );
    token.grant_minter(Sender(owner), bob);
    token.update_token_commission(
        Sender(bob),
        token_id,
        "artist".to_string(),
        owner,
        "UPDATE".to_string(),
        "12".to_string(),
    );
}

#[test]
#[should_panic]
fn test_token_commission_update_from_owner() {
    let (env, token, owner) = deploy();
    let ali = env.next_user();
    let token_id = TokenId::from("123456");
    let token_meta = meta::big_cask();
    let token_gauge = gauge::alchol_gauge();
    let token_warehouse = warehouse::west_warehouse();
    let token_commission = commission::commission(
        vec!["artist".to_string()],
        vec![ali.into()],
        vec!["10".to_string()],
    );

    token.mint_copies(
        Sender(owner),
        ali,
        Some(vec![token_id.clone()]),
        token_meta,
        token_gauge,
        token_warehouse,
        token_commission,
        1,
    );
    token.update_token_commission(
        Sender(ali),
        token_id,
        "artist".to_string(),
        owner,
        "UPDATE".to_string(),
        "12".to_string(),
    );
}
