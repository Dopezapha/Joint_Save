use soroban_sdk::{token, Address, Env, Vec};

/// Deposit into a Stellar AMM liquidity pool.
/// Transfers `amount` to the pool contract and invokes `deposit`.
pub fn amm_deposit(env: &Env, pool: &Address, token: &Address, amount: i128, to: &Address) -> i128 {
    let client = token::Client::new(env, token);
    client.transfer(&env.current_contract_address(), pool, &amount);

    let _: Vec<soroban_sdk::Val> = env.invoke_contract(
        pool,
        &soroban_sdk::symbol_short!("deposit"),
        soroban_sdk::vec![
            env,
            soroban_sdk::IntoVal::into_val(to, env),
            soroban_sdk::IntoVal::into_val(&amount, env),
        ],
    );

    amount
}

/// Withdraw from a Stellar AMM liquidity pool.
/// Returns the principal amount withdrawn.
pub fn amm_withdraw(env: &Env, pool: &Address, token: &Address, amount: i128, to: &Address) -> i128 {
    let _: Vec<soroban_sdk::Val> = env.invoke_contract(
        pool,
        &soroban_sdk::symbol_short!("withdraw"),
        soroban_sdk::vec![
            env,
            soroban_sdk::IntoVal::into_val(to, env),
            soroban_sdk::IntoVal::into_val(&amount, env),
        ],
    );

    // Transfer proceeds back from pool to our contract
    let client = token::Client::new(env, token);
    client.transfer(pool, &env.current_contract_address(), &amount);

    amount
}
