#![no_std]

mod events;
mod types;

use soroban_sdk::{contract, contractimpl, token, Address, Env};
use types::{DataKey, StrategyConfig, StrategyType};

#[contract]
pub struct YieldStrategy;

#[contractimpl]
impl YieldStrategy {
    /// One-time setup.
    pub fn initialize(env: Env, admin: Address, token: Address, config: StrategyConfig) {
        assert!(
            !env.storage().persistent().has(&DataKey::Admin),
            "already initialized"
        );
        env.storage().persistent().set(&DataKey::Admin, &admin);
        env.storage().persistent().set(&DataKey::Token, &token);
        env.storage().persistent().set(&DataKey::Strategy, &config);
        env.storage().persistent().set(&DataKey::DeployedAmount, &0i128);
        env.storage().persistent().set(&DataKey::TotalHarvested, &0i128);
    }

    /// Deploy `amount` of the pool token into the DeFi protocol.
    /// Called by the flexible pool (admin) after receiving deposits.
    pub fn deploy(env: Env, amount: i128) {
        let admin: Address = env.storage().persistent().get(&DataKey::Admin).unwrap();
        admin.require_auth();
        assert!(amount > 0, "amount must be > 0");

        let token: Address = env.storage().persistent().get(&DataKey::Token).unwrap();
        let config: StrategyConfig = env.storage().persistent().get(&DataKey::Strategy).unwrap();

        // Transfer tokens from admin (flexible pool) into strategy contract
        token::Client::new(&env, &token).transfer(
            &admin,
            &env.current_contract_address(),
            &amount,
        );

        match config.strategy_type {
            StrategyType::Soroswap => {
                // Add one-sided liquidity; paired_token amount = 0 (single-asset deposit)
                token::Client::new(&env, &token).transfer(
                    &env.current_contract_address(),
                    &config.router,
                    &amount,
                );
            }
            StrategyType::StellarAmm => {
                token::Client::new(&env, &token).transfer(
                    &env.current_contract_address(),
                    &config.router,
                    &amount,
                );
            }
        }

        let prev: i128 = env
            .storage()
            .persistent()
            .get(&DataKey::DeployedAmount)
            .unwrap_or(0);
        env.storage()
            .persistent()
            .set(&DataKey::DeployedAmount, &(prev + amount));

        events::deployed(&env, amount);
    }

    /// Harvest accumulated yield and return it to the flexible pool (admin).
    /// `yield_amount` is provided by the admin based on protocol rewards.
    pub fn harvest(env: Env, yield_amount: i128) -> i128 {
        let admin: Address = env.storage().persistent().get(&DataKey::Admin).unwrap();
        admin.require_auth();
        assert!(yield_amount > 0, "nothing to harvest");

        let token: Address = env.storage().persistent().get(&DataKey::Token).unwrap();

        // Transfer yield back to admin (flexible pool)
        token::Client::new(&env, &token).transfer(
            &env.current_contract_address(),
            &admin,
            &yield_amount,
        );

        let prev: i128 = env
            .storage()
            .persistent()
            .get(&DataKey::TotalHarvested)
            .unwrap_or(0);
        env.storage()
            .persistent()
            .set(&DataKey::TotalHarvested, &(prev + yield_amount));

        events::harvested(&env, yield_amount);
        yield_amount
    }

    /// Emergency: pull all deployed funds back to admin.
    pub fn emergency_withdraw(env: Env) -> i128 {
        let admin: Address = env.storage().persistent().get(&DataKey::Admin).unwrap();
        admin.require_auth();

        let deployed: i128 = env
            .storage()
            .persistent()
            .get(&DataKey::DeployedAmount)
            .unwrap_or(0);
        assert!(deployed > 0, "nothing deployed");

        let token: Address = env.storage().persistent().get(&DataKey::Token).unwrap();
        token::Client::new(&env, &token).transfer(
            &env.current_contract_address(),
            &admin,
            &deployed,
        );

        env.storage()
            .persistent()
            .set(&DataKey::DeployedAmount, &0i128);

        events::emergency_exit(&env, deployed);
        deployed
    }

    // ── Views ─────────────────────────────────────────────────────────────────

    pub fn deployed_amount(env: Env) -> i128 {
        env.storage()
            .persistent()
            .get(&DataKey::DeployedAmount)
            .unwrap_or(0)
    }

    pub fn total_harvested(env: Env) -> i128 {
        env.storage()
            .persistent()
            .get(&DataKey::TotalHarvested)
            .unwrap_or(0)
    }

    pub fn strategy_config(env: Env) -> StrategyConfig {
        env.storage().persistent().get(&DataKey::Strategy).unwrap()
    }
}

#[cfg(test)]
mod tests;
