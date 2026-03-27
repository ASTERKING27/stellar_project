#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, Vec, token};

// 1. Defining the variables we want to save permanently on the blockchain
#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,           // The group creator
    Token,           // The asset used (e.g., USDC)
    Contribution,    // The monthly base payment
    Members,         // List of participant wallets
    Cycle,           // Tracks which month we are in
}

#[contract]
pub struct ChitFundContract;

// 2. The functions that manipulate the data
#[contractimpl]
impl ChitFundContract {
    
    // The "Constructor" - runs once when the contract is deployed
    pub fn init(
        env: Env,
        admin: Address,
        token: Address,
        contribution: i128,
        members: Vec<Address>,
    ) {
        // Ensure only the admin can trigger this setup
        admin.require_auth(); 
        
        // Save all the initial group settings into the contract's storage
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Token, &token);
        env.storage().instance().set(&DataKey::Contribution, &contribution);
        env.storage().instance().set(&DataKey::Members, &members);
        env.storage().instance().set(&DataKey::Cycle, &1u32);
    }

    // A simple "getter" function to check what month the group is on
    pub fn get_cycle(env: Env) -> u32 {
        env.storage().instance().get(&DataKey::Cycle).unwrap_or(0)
    }

    // Allows a member to pay their monthly contribution into the pool
    pub fn deposit(env: Env, member: Address) {
        // 1. Security Check: Ensure the person calling this function actually signed it.
        member.require_auth();

        // 2. Fetch the rules we saved during 'init'
        let token_address: Address = env.storage().instance().get(&DataKey::Token).unwrap();
        let amount: i128 = env.storage().instance().get(&DataKey::Contribution).unwrap();

        // 3. Create a "client" to talk to the token (like USDC or native XLM)
        let token_client = token::Client::new(&env, &token_address);

        // 4. Execute the transfer: Move money from the member's wallet into the contract itself
        token_client.transfer(&member, &env.current_contract_address(), &amount);
    }
    // Executes the payout to the winner of the current cycle
    pub fn payout(env: Env, winner: Address) {
        // 1. Security Check: Only the Admin should be able to trigger the payout for now
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        admin.require_auth();

        // 2. Setup the token client
        let token_address: Address = env.storage().instance().get(&DataKey::Token).unwrap();
        let token_client = token::Client::new(&env, &token_address);

        // 3. Check how much money the contract currently holds (the total pool)
        let contract_balance = token_client.balance(&env.current_contract_address());

        // 4. Send the entire balance to the winner
        token_client.transfer(&env.current_contract_address(), &winner, &contract_balance);

        // 5. Move the group to the next cycle (Month 1 becomes Month 2)
        let current_cycle: u32 = env.storage().instance().get(&DataKey::Cycle).unwrap();
        env.storage().instance().set(&DataKey::Cycle, &(current_cycle + 1));
    }
}