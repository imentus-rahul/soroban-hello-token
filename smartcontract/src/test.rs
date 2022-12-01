#![cfg(test)]
extern crate std;
use super::testutils::{register_test_contract as register_hello_contract, ContractTest};
use super::token::{Client as Token, TokenMetadata}; // change Token == TokenClient

use super::{Contract, ContractClient}; // directly from smart contract

use rand::{thread_rng, RngCore};

use ed25519_dalek::Keypair;

use soroban_auth::{Identifier, Signature};

use soroban_sdk::{
    bytesn, symbol,
    testutils::{Accounts, Ledger, Logger},
    vec, AccountId, BigInt, BytesN, Env, IntoVal,
};

// helper functions for generating hello world contract id and token contract id in test environment of soroban

// helper function just to generate random ids
fn generate_contract_id() -> [u8; 32] {
    let mut id: [u8; 32] = Default::default();
    thread_rng().fill_bytes(&mut id); // generating some random hello world contract id
    id
}

// invoke method: "hello" and "world"
#[test]
fn test() {
    let env = Env::default();
    let contract_id = env.register_contract(None, Contract);
    let client = ContractClient::new(&env, &contract_id);

    let some_u64: u64 = 3;

    let words = client.hello(&symbol!("Dev"), &some_u64); // passing multiple params
    let logs = env.logger().all();
    assert_eq!(
        logs,
        std::vec!["fn hello params: to: Symbol(Dev) param2: Object(U64(6))"]
    );
    std::println!("{}", logs.join("\n"));

    assert_eq!(words, vec![&env, symbol!("Hello"), symbol!("Dev"),]); // output check returned from "hello" method invocation

    let words = client.world();
    assert_eq!(words, vec![&env, symbol!("Hello"),]);
}

// Step 1: Register the hello world smart contract in test cases -> use the helper function from testutils.rs
// fn register_test_contract

// Step 2: Register the token smart contract for a new asset

// invoke method: "initialize"
#[test]
fn test2() {
    let env = Env::default();
    // Deploy the hello world contract here
    let hello_contract_id = env.register_contract(None, Contract); // registering hello world smart contract
    let hello_client = ContractClient::new(&env, hello_contract_id.clone()); // creating hello world client

    // Way 04
    let (vault_account_id, vault_account_sign) = soroban_auth::testutils::ed25519::generate(&env);

    // this is the target amount; whenever user invokes deposit method; he/she pays 10 token to above vault_account
    let amount: BigInt = BigInt::from_u32(&env, 10);

    // Identifier::Account(AccountId)
    let token_admin1 = env.accounts().generate_and_create();
    let token_admin1_id = Identifier::Account(token_admin1.clone());

    // Identifier::Ed25519(BytesN<32>)
    let (token_admin_id, token_admin_sign) = soroban_auth::testutils::ed25519::generate(&env);

    // token contract id
    let id = generate_contract_id(); // generates random id which is later used by register_contract_token
                                     // built in function: register_contract_token == register_test_contract_token
    env.register_contract_token(&BytesN::from_array(&env, &id));
    let token_client = Token::new(&env, id);

    // let token_contract_id = env.register_contract_wasm(None, token::WASM);
    // let token_client = Token::new(&env, token_contract_id.clone());

    std::println!("L398 about to invoke init method of smart contract");

    token_client.init(
        &token_admin_id, // token_admin1_id // token_admin_id
        &TokenMetadata {
            name: "USDC Token".into_val(&env),
            symbol: "USDC".into_val(&env),
            decimals: 7,
        },
    );

    std::println!("L409 invoked init method of smart contract");

    // initialize method can be invoked by anyone
    hello_client.initialize(&vault_account_id, &amount, &BytesN::from_array(&env, &id));
    std::println!("initialize method invoked, mutates a state in soroban env");

    let assert_data = hello_client.recipient();
    std::println!("assert_data - recipient {:?}", assert_data);
    assert_eq!(assert_data, vault_account_id.clone()); //Identifier Type

    let assert_data = hello_client.target();
    std::println!("assert_data - target {:?}", assert_data);
    assert_eq!(assert_data, BigInt::from_u32(&env, 10));

    let assert_data = hello_client.tokenid();
    std::println!("assert_data - tokenid {:?}", assert_data);
    // assert_eq!(assert_data, &token_id); // can't compare `soroban_sdk::BytesN<32>` with `&soroban_sdk::BytesN<32>` the trait `PartialEq<&soroban_sdk::BytesN<32>>` is not implemented for `soroban_sdk::BytesN<32>` the following other types implement trait `PartialEq<Rhs>`

    // next mint some tokens to some random user accounts and further deposit those tokens to vault account using "deposit" method of hello_contract

    // create random user
    let user1 = env.accounts().generate_and_create();
    let user1_id = Identifier::Account(user1.clone());

    let (user2_id, user2_sign) = soroban_auth::testutils::ed25519::generate(&env);

    // ----------------------------------------------------------------------------------------------------------------------- \\

    // // token_client + "mint" + with_source_account
    // // minting some tokens in soroban environment
    // // minting 10 tokens to user 1
    // token_client.with_source_account(&token_admin1).mint(
    //     // pass AccountId==token_admin
    //     &Signature::Invoker,
    //     &BigInt::zero(&env),
    //     // &token_client.nonce(&Signature::Invoker.identifier(&env)), // this don't work
    //     &user1_id,
    //     &BigInt::from_u32(&env, 10),
    // );

    // token_client + "mint" + sign
    let sig = soroban_auth::testutils::ed25519::sign(
        &env,
        &token_admin_sign,
        &BytesN::from_array(&env, &id), // token_contract_id BytesN<32>
        symbol!("mint"),
        (
            &token_admin_id,
            &BigInt::zero(&env),
            &user2_id,                    //user2_id // user1_id
            &BigInt::from_u32(&env, 100), //  0: "Failed ED25519 verification" => when you sign with param as "10" but invoking on "100"
        ),
    );

    // minting tokens using token_client and signatures
    token_client.mint(
        // pass AccountId==token_admin
        &sig,
        &BigInt::zero(&env),
        // &token_client.nonce(&Signature::Invoker.identifier(&env)), // this don't work
        &user2_id,
        &BigInt::from_u32(&env, 100),
    );

    // check all balances here after minting
    std::println!(
        "L504 After Minting - token_client.balance(&user2_id): {}",
        token_client.balance(&user2_id)
    );
    std::println!(
        "After Minting - token_client.balance(&vault_account_id): {}",
        token_client.balance(&vault_account_id)
    );

    // ----------------------------------------------------------------------------------------------------------------------- \\

    // env.set_source_account(&user1);

    std::println!("x1");

    // hello_client + "mint" + with_source_account
    // HAVING TROUBLES INVOKING MINT METHOD OF TOKEN CONTRACT FROM HELLO WORLD CONTRACT
    // retry with hello_contract mint method
    // hello_client
    //     .with_source_account(&token_admin)
    //     .mint(&user1_id);

    std::println!("x2");

    // hello_client + "mint" + sign
    // this don't work
    let sig = soroban_auth::testutils::ed25519::sign(
        &env,
        &token_admin_sign,
        &hello_contract_id,
        symbol!("mint"),
        // (&hello_contract_id, &user1_id),
        (&token_admin_id, &BigInt::zero(&env), &user1_id, &BigInt::from_u32(&env, 10)),
    );
    std::println!("x3");

    hello_client.testmint(&sig, &user1_id); // this ain't working
    std::println!(" if x4 prints, problem is resolved");

    std::println!("x4");

    // let logs = env.logger().all();
    // std::println!("After Minting 2 from Hello Client: {:?}", logs);

    // check all balances here after minting
    std::println!(
        "After Minting 2 from Hello Client - token_client.balance(&user1_id): {}",
        token_client.balance(&user1_id)
    );
    std::println!(
        "After Minting 2 from Hello Client - token_client.balance(&vault_account_id): {}",
        token_client.balance(&vault_account_id)
    );

    // ----------------------------------------------------------------------------------------------------------------------- \\

    // // token_client + with_source_account + "xfer" 10 tokens
    // // this works
    // // trying direct transfer in soroban environment using token client
    // token_client.with_source_account(&user1).xfer(
    //     // pass AccountId==token_admin
    //     &Signature::Invoker,
    //     &BigInt::zero(&env),
    //     &vault_account_id,
    //     &BigInt::from_u32(&env, 10),
    // );

    // signature for token_client + sig + "xfer" 5 tokens
    let sig = soroban_auth::testutils::ed25519::sign(
        &env,
        &user2_sign,
        &BytesN::from_array(&env, &id), // token_contract_id BytesN<32>
        symbol!("xfer"),
        (
            &user2_id,
            &BigInt::zero(&env),
            &vault_account_id,
            &BigInt::from_u32(&env, 5),
        ),
    );

    // xfering tokens using token_client + sig + "xfer" 5 tokens
    token_client.xfer(
        // pass AccountId==token_admin
        &sig,
        &BigInt::zero(&env),
        // &token_client.nonce(&Signature::Invoker.identifier(&env)), // this don't work
        &vault_account_id,
        &BigInt::from_u32(&env, 5),
    );

    // check all balances here after transfer
    std::println!(
        "L595: token_client After Transfer - token_client.balance(&user2_id): {}",
        token_client.balance(&user2_id)
    );
    std::println!(
        "token_client After Transfer - token_client.balance(&vault_account_id): {}",
        token_client.balance(&vault_account_id)
    );

    // ----------------------------------------------------------------------------------------------------------------------- \\

    std::println!("y1");

    // // This is NOT APPLICABLE
    // // hello_client + "xfer" + with_source_account
    // // HAVING TROUBLES INVOKING XFER METHOD OF TOKEN CONTRACT FROM HELLO WORLD CONTRACT
    // // further deposit 10 tokens to vault account with user1 account
    // hello_client.with_source_account(&user1).deposit();
    // // check all balances here after depositing
    // std::println!("L556: After Depositing - token_client.balance(&user1_id): {}", token_client.balance(&user1_id));
    // std::println!("After Depositing - token_client.balance(&vault_account_id): {}", token_client.balance(&vault_account_id));

    std::println!("y2");

    // // hello_client + "deposit" + sign
    // this don't work
    let sig = soroban_auth::testutils::ed25519::sign(
        &env,
        &user2_sign,
        &hello_contract_id,
        symbol!("deposit"),
        (&user2_id,),
    );
    std::println!("y3");

    // hello_client.deposit(&sig); // this ain't working
    // std::println!(" if y4 prints, problem is resolved");

    std::println!("y4");

    // check all balances here after transfer
    std::println!(
        "hello_client After Transfer 2 - token_client.balance(&user2_id): {}",
        token_client.balance(&user2_id)
    );
    std::println!(
        "hello_client After Transfer 2 - token_client.balance(&vault_account_id): {}",
        token_client.balance(&vault_account_id)
    );

    // ----------------------------------------------------------------------------------------------------------------------- \\

    let assert_data = hello_client.with_source_account(&user1).name();
    std::println!(
        "assert_data - name of token - checking token client in smart contract {:?}",
        assert_data
    ); // this works that means we don't have any issue with similar token client in "deposit" method

    let logs = env.logger().all();
    std::println!("After invoking name method from Hello Client: {:?}", logs);

    std::println!(
        "Balance using hello contract client - user2 {}",
        hello_client.with_source_account(&user1).balance(&user2_id)
    );
}