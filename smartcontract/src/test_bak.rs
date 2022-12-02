#![cfg(test)]

extern crate std;

use soroban_sdk::{bigint, bytesn, symbol, BigInt, BytesN, Env, IntoVal};

use super::token;
use super::{TestMintContract, TestMintContractClient};

use rand::{thread_rng, RngCore};

// helper function just to generate random ids
fn generate_contract_id() -> [u8; 32] {
    let mut id: [u8; 32] = Default::default();
    thread_rng().fill_bytes(&mut id); // generating some random hello world contract id
    id
}

#[test]
fn test_mint() {
    let e: Env = Env::default();
    let (token_admin_id, token_admin_sign) = soroban_auth::testutils::ed25519::generate(&e);
    let (user1_id, user1_sign) = soroban_auth::testutils::ed25519::generate(&e);
    let (vault_account_id, vault_account_sign) = soroban_auth::testutils::ed25519::generate(&e);

    // // token contract id BytesN<32>
    // let token_id = e.register_contract_token(&bytesn!(
    //     &e,
    //     0x25a93e3c5b3de9f9dbedb86cd6f4c34e2e98df1cb50935b4379d07369b397ecb
    // ));

    // id [u8;32]
    let id = generate_contract_id(); // generates random id which is later used by register_contract_token
    let token_id = BytesN::from_array(&e, &id);
    // built in function: register_contract_token == register_test_contract_token
    e.register_contract_token(&token_id);

    // token client
    let token_client = token::Client::new(&e, &token_id);

    let hello_id = e.register_contract(None, TestMintContract);
    let hello_client = TestMintContractClient::new(&e, &hello_id);

    // initialize method can be invoked by anyone
    hello_client.initialize(&vault_account_id, &BigInt::from_u32(&e, 10), &token_id);
    std::println!("initialize method invoked, mutates a state in soroban env");

    let assert_data = hello_client.recipient();
    std::println!("assert_data - recipient {:?}", assert_data);
    assert_eq!(assert_data, vault_account_id.clone()); //Identifier Type

    let assert_data = hello_client.target();
    std::println!("assert_data - target {:?}", assert_data);
    assert_eq!(assert_data, BigInt::from_u32(&e, 10));

    let assert_data = hello_client.tokenid();
    std::println!("assert_data - tokenid {:?}", assert_data);

    // custom nonce using token_admin
    let nonce = token_client.nonce(&token_admin_id);
    std::println!("🚀 ~ nonce {:?}", nonce);

    token_client.init(
        &token_admin_id,
        &token::TokenMetadata {
            name: "USD coin".into_val(&e),
            symbol: "USDC".into_val(&e),
            decimals: 7,
        },
    );

    // create sign for the ultimate contract that'll consume the sign
    let mint_sig = soroban_auth::testutils::ed25519::sign(
        &e,
        &token_admin_sign,
        &token_id,
        symbol!("mint"),
        (&token_admin_id, &nonce, &user1_id, &hello_client.target()),
    );

    hello_client.t_mint(&mint_sig, &hello_client.target(), &user1_id);

    // check all balances here after minting
    std::println!(
        "After Minting using hello_client - token.balance(&user1_id): {}",
        token_client.balance(&user1_id)
    );

    let nonce2 = token_client.nonce(&token_admin_id);
    std::println!("🚀 ~ nonce2 {:?}", nonce2);

    let mint_sig2 = soroban_auth::testutils::ed25519::sign(
        &e,
        &token_admin_sign,
        &token_id,
        symbol!("mint"),
        (&token_admin_id, &nonce2, &user1_id, &hello_client.target()),
    );

    // minting tokens using token_client and signatures
    token_client.mint(&mint_sig2, &nonce2, &user1_id, &hello_client.target());
    // check all balances here after minting
    std::println!(
        "After Minting using token_client - token.balance(&user1_id): {}",
        token_client.balance(&user1_id)
    );
}
