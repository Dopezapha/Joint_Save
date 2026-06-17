#![cfg(test)]

use super::{YieldStrategy, YieldStrategyClient};
use crate::types::{StrategyConfig, StrategyType};
use soroban_sdk::{testutils::Address as _, token, Address, Env};

fn setup() -> (Env, YieldStrategyClient<'static>, Address, Address, Address) {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, YieldStrategy);
    let client = YieldStrategyClient::new(&env, &contract_id);

    let token_admin = Address::generate(&env);
    let token_contract = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_address = token_contract.address();
    let token_client = token::StellarAssetClient::new(&env, &token_address);

    let admin = Address::generate(&env);
    let router = Address::generate(&env);

    // Mint tokens to admin (acts as flexible pool)
    token_client.mint(&admin, &10_000i128);
    // Mint tokens to strategy contract for harvest simulation
    token_client.mint(&contract_id, &1_000i128);

    let config = StrategyConfig {
        strategy_type: StrategyType::Soroswap,
        router: router.clone(),
        paired_token: token_address.clone(),
    };
    client.initialize(&admin, &token_address, &config);

    (env, client, admin, token_address, router)
}

#[test]
fn test_deploy_and_deployed_amount() {
    let (_env, client, _admin, _token, _router) = setup();
    client.deploy(&500i128);
    assert_eq!(client.deployed_amount(), 500);
}

#[test]
fn test_harvest_yield() {
    let (_env, client, _admin, _token, _router) = setup();
    client.deploy(&500i128);
    let harvested = client.harvest(&100i128);
    assert_eq!(harvested, 100);
    assert_eq!(client.total_harvested(), 100);
}

#[test]
fn test_emergency_withdraw() {
    let (_env, client, _admin, _token, _router) = setup();
    client.deploy(&500i128);
    let withdrawn = client.emergency_withdraw();
    assert_eq!(withdrawn, 500);
    assert_eq!(client.deployed_amount(), 0);
}

#[test]
#[should_panic(expected = "already initialized")]
fn test_double_initialize() {
    let (_env, client, admin, token_address, router) = setup();
    let config = StrategyConfig {
        strategy_type: StrategyType::StellarAmm,
        router,
        paired_token: token_address.clone(),
    };
    client.initialize(&admin, &token_address, &config);
}

#[test]
#[should_panic(expected = "nothing deployed")]
fn test_emergency_withdraw_nothing() {
    let (_env, client, _admin, _token, _router) = setup();
    client.emergency_withdraw();
}

#[test]
fn test_stellar_amm_strategy() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, YieldStrategy);
    let client = YieldStrategyClient::new(&env, &contract_id);

    let token_admin = Address::generate(&env);
    let token_contract = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_address = token_contract.address();
    let token_client = token::StellarAssetClient::new(&env, &token_address);

    let admin = Address::generate(&env);
    let router = Address::generate(&env);
    token_client.mint(&admin, &5_000i128);
    token_client.mint(&contract_id, &500i128);

    let config = StrategyConfig {
        strategy_type: StrategyType::StellarAmm,
        router: router.clone(),
        paired_token: token_address.clone(),
    };
    client.initialize(&admin, &token_address, &config);
    client.deploy(&1_000i128);
    assert_eq!(client.deployed_amount(), 1_000);
}
