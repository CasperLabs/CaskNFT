use casper_engine_test_support::AccountHash;
use casper_types::{Key, U256};
use test_env::{Sender, TestEnv};

use crate::cask_collectible_instance::{CaskCollectibleInstance, Commission, Meta, TokenId};

const NAME: &str = "CaskCollectibleNFT";
const SYMBOL: &str = "CCNFT";

mod meta {
    use super::Meta;
    pub fn contract_meta() -> Meta {
        let mut meta = Meta::new();
        meta.insert("image".to_string(), "img_1.png".to_string());
        meta
    }

    pub fn img_2() -> Meta {
        let mut meta = Meta::new();
        meta.insert("image".to_string(), "img_2.png".to_string());
        meta
    }

    pub fn img_3() -> Meta {
        let mut meta = Meta::new();
        meta.insert("image".to_string(), "img_3.png".to_string());
        meta
    }
}

mod commission {
    use casper_types::Key;

    use super::Commission;
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
            commission.insert(format!("{}_account", property.clone()), account.to_string());
            commission.insert(format!("{}_rate", property.clone()), rate.clone());
        }
        commission
    }
}

fn deploy() -> (TestEnv, CaskCollectibleInstance, AccountHash) {
    let env = TestEnv::new();
    let owner = env.next_user();
    let token = CaskCollectibleInstance::new(
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
fn test_mint_from_minter() {
    let (env, token, owner) = deploy();
    let ali = env.next_user();
    let bob = env.next_user();
    let token_id = TokenId::from("custom_token_id");
    let token_meta = meta::img_2();
    let token_commission = commission::commission(
        vec!["artist".to_string(), "broker".to_string()],
        vec![ali.into(), bob.into()],
        vec!["10".to_string(), "12".to_string()],
    );

    token.mint(
        Sender(owner),
        bob,
        Some(vec![token_id.clone()]),
        vec![token_meta.clone()],
        vec![token_commission.clone()],
    );

    let user_token_meta = token.token_meta(token_id.clone());
    assert_eq!(user_token_meta.unwrap(), token_meta);

    let user_token_commission = token.token_commission(token_id.clone());
    assert_eq!(user_token_commission.unwrap(), token_commission);

    let first_user_token = token.get_token_by_index(Key::Account(bob), U256::zero());
    assert_eq!(first_user_token, Some(token_id));
}

#[test]
#[should_panic]
fn test_mint_with_wrong_arguments() {
    let (env, token, owner) = deploy();
    let ali = env.next_user();
    let bob = env.next_user();
    let token_meta = meta::img_3();

    token.mint(
        Sender(ali),
        bob,
        None,
        vec![token_meta],
        vec![],
    );
}

#[test]
#[should_panic]
fn test_mint_from_non_minter() {
    let (env, token, _) = deploy();
    let ali = env.next_user();
    let bob = env.next_user();
    let token_id = TokenId::from("custom_token_id");
    let token_meta = meta::img_3();
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
        vec![token_commission],
    );
}

#[test]
fn test_burn_from_non_minter() {
    let (env, token, owner) = deploy();
    let ali = env.next_user();
    let bob = env.next_user();
    let token_meta = meta::img_2();
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
    let token_meta = meta::img_2();
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
    let token_meta = meta::img_2();
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
    let token_meta = meta::img_2();
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
    token.transfer_from(Sender(bob), ali, bob, vec![first_ali_token.unwrap()]);
}

#[test]
fn test_transfer() {
    let (env, token, owner) = deploy();
    let ali = env.next_user();
    let bob = env.next_user();
    let token_meta = meta::img_2();
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
    let token_meta = meta::img_2();
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
        token_commission.clone(),
        1,
    );

    let user_token_meta = token.token_meta(token_id.clone());
    assert_eq!(user_token_meta.unwrap(), token_meta);

    let user_token_commission = token.token_commission(token_id.clone());
    assert_eq!(user_token_commission.unwrap(), token_commission);

    let first_user_token = token.get_token_by_index(Key::Account(ali), U256::zero());
    assert_eq!(first_user_token, Some(token_id));
}
