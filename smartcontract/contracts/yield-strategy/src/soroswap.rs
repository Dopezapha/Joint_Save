use soroban_sdk::{token, Address, Env, Vec};

/// Add liquidity to a Soroswap pair.
/// Returns the LP tokens received (estimated as the minimum of the two amounts).
/// In a real deployment the router would be called via `invoke_contract`.
pub fn add_liquidity(
    env: &Env,
    router: &Address,
    token_a: &Address,
    token_b: &Address,
    amount_a: i128,
    amount_b: i128,
    to: &Address,
) -> i128 {
    // Transfer token_a from caller (the strategy contract) to router
    let client_a = token::Client::new(env, token_a);
    client_a.transfer(&env.current_contract_address(), router, &amount_a);

    // Transfer token_b from caller to router
    if amount_b > 0 {
        let client_b = token::Client::new(env, token_b);
        client_b.transfer(&env.current_contract_address(), router, &amount_b);
    }

    // Invoke router add_liquidity — placeholder inter-contract call pattern
    let _: Vec<soroban_sdk::Val> = env.invoke_contract(
        router,
        &soroban_sdk::symbol_short!("add_liq"),
        soroban_sdk::vec![
            env,
            soroban_sdk::IntoVal::into_val(token_a, env),
            soroban_sdk::IntoVal::into_val(token_b, env),
            soroban_sdk::IntoVal::into_val(&amount_a, env),
            soroban_sdk::IntoVal::into_val(&amount_b, env),
            soroban_sdk::IntoVal::into_val(to, env),
        ],
    );

    // Return deployed principal (amount_a represents our asset stake)
    amount_a
}

/// Remove liquidity and return proceeds to `to`.
pub fn remove_liquidity(
    env: &Env,
    router: &Address,
    token_a: &Address,
    token_b: &Address,
    lp_amount: i128,
    to: &Address,
) -> i128 {
    let _: Vec<soroban_sdk::Val> = env.invoke_contract(
        router,
        &soroban_sdk::symbol_short!("rem_liq"),
        soroban_sdk::vec![
            env,
            soroban_sdk::IntoVal::into_val(token_a, env),
            soroban_sdk::IntoVal::into_val(token_b, env),
            soroban_sdk::IntoVal::into_val(&lp_amount, env),
            soroban_sdk::IntoVal::into_val(to, env),
        ],
    );

    // Return the amount we get back (lp_amount as proxy for principal)
    lp_amount
}
