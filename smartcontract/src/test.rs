// #[cfg(test)]
#![cfg(test)]

// // try to get values from dotenv
// #[macro_use]
// extern crate dotenv_codegen;

extern crate std;

// importing from additional testutils files
use super::testutils::{register_test_contract as register_hello_contract, ContractTest};
use super::token::{Client as Token, TokenMetadata}; // change Token == TokenClient

use super::{Contract, ContractClient}; // directly from smart contract

use rand::{thread_rng, RngCore};

use ed25519_dalek::Keypair;

// use soroban_auth::testutils::ed25519::Identifier;
use soroban_auth::{Identifier, Signature};

use soroban_sdk::{
    bytesn, symbol,
    testutils::{Accounts, Ledger, Logger},
    vec, AccountId, BigInt, BytesN, Env, IntoVal,
};

// use crate::test::std::string::ToString;

// should keep for future references
// mod ed25519_utils {
//     extern crate ed25519_dalek;
//     extern crate std;

//     use soroban_sdk::Env;

//     use soroban_sdk::{testutils::ed25519::Sign, Bytes, IntoVal};

//     use soroban_auth::{
//         testutils::ed25519::Identifier, Identifier as IdentifierValue, SignaturePayload,
//     };

//     use core::fmt::Debug;

//     use stellar_strkey::*;

//     pub fn generate(
//         env: &Env,
//     ) -> (
//         IdentifierValue,
//         Bytes,
//         impl soroban_auth::testutils::ed25519::Identifier
//             + Sign<SignaturePayload, Signature = [u8; 64]>
//             + Debug,
//     ) {
//         let signer = ed25519_dalek::Keypair::generate(&mut rand::thread_rng());
//         (
//             IdentifierValue::Ed25519(signer.public.as_bytes().into_val(env)),
//             signer.to_bytes().into_val(env),
//             signer,
//         )
//     }

// // build keypair fn generates keypair from your string private keypair
//     pub fn build_kp(
//         env: &Env,
//         public: &[u8],
//         secret: &[u8],
//     ) -> (
//         IdentifierValue,
//         impl soroban_auth::testutils::ed25519::Identifier
//             + Sign<SignaturePayload, Signature = [u8; 64]>
//             + Debug,
//     ) {
//         let kp = ed25519_dalek::Keypair {
//             secret: ed25519_dalek::SecretKey::from_bytes(secret).unwrap(),
//             public: ed25519_dalek::PublicKey::from_bytes(public).unwrap(),
//         };

//         (kp.identifier(env), kp)
//     }

//     pub fn log_pubkey(id: IdentifierValue) {
//         let user_account_id = match id {
//             IdentifierValue::Ed25519(bytes) => bytes,
//             _ => panic!("not ed25519"),
//         };

//         std::println!(
//             "{:?}",
//             &Strkey::PublicKeyEd25519(StrkeyPublicKeyEd25519(user_account_id.to_array()))
//                 .to_string(),
//         )
//     }

//     pub fn log_secret(secret: [u8; 32]) {
//         std::println!(
//             "{:?}",
//             &Strkey::PrivateKeyEd25519(StrkeyPrivateKeyEd25519(secret)).to_string(),
//         )
//     }

//     // pub fn decode_pub(public: String) -> [u8; 32] {
//     //     let pub_key = StrkeyPublicKeyEd25519::from_string(&public).unwrap();
//     //     pub_key.0
//     // }

//     // pub fn decode_secret(secret: String) -> [u8; 32] {
//     //     let secret_key = StrkeyPrivateKeyEd25519::from_string(&secret).unwrap();
//     //     secret_key.0
//     // }
// }

// #[derive(Clone)]
// #[contracttype]
// pub enum DataKey {
//     Recipient, // common vault address where everyone will deposit tokens
//     Target,    // target amount of tokens that an invoker will transfer to
//     TokenId,   // token program id on top of of soroban
// }

// helper functions for generating hello world contract id and token contract id in test environment of soroban

// helper function just to generate random ids
fn generate_contract_id() -> [u8; 32] {
    let mut id: [u8; 32] = Default::default();
    thread_rng().fill_bytes(&mut id); // generating some random hello world contract id
    id
}

// here we are generating a new token by invoking "init" method and registering it with a new token contract id
// here the token we're generating is origin of soroban environment and not the classic stellar token
fn create_token_contract(e: &Env, admin: &AccountId) -> (BytesN<32>, Token) {
    let id = generate_contract_id(); // generates random id which is later used by register_contract_token
                                     // built in function: register_contract_token == register_test_contract_token
    e.register_contract_token(&BytesN::from_array(e, &id));
    let token = Token::new(e, id);
    // decimals, name, symbol don't matter in tests
    token.init(
        &Identifier::Account(admin.clone()),
        &TokenMetadata {
            name: "EBI Token".into_val(e),
            symbol: "EBI".into_val(e),
            decimals: 7,
        },
    );
    (BytesN::from_array(e, &id), token)
}

// // here we are generating a new token by invoking "init" method and registering it with a new token contract id
// // here the token we're generating is origin of soroban environment and not the classic stellar token
// fn create_token_contract_ed25519(e: &Env, admin: &Identifier) -> (BytesN<32>, Token) {
//     let id = generate_contract_id(); // generates random id which is later used by register_contract_token
//                                      // built in function: register_contract_token == register_test_contract_token
//     e.register_contract_token(&BytesN::from_array(e, &id));
//     let token = Token::new(e, id);
//     // decimals, name, symbol don't matter in tests
//     token.init(
//         &admin,
//         &TokenMetadata {
//             name: "EBI Token".into_val(e),
//             symbol: "EBI".into_val(e),
//             decimals: 7,
//         },
//     );
//     (BytesN::from_array(e, &id), token)
// }

// here we're registering the hello_world smart contract id in test env
// till this point we have a total of 2 contract ids, token contract id and hello world contract id
fn create_hello_contract(
    e: &Env,
    recipient: &AccountId,
    target_amount: &BigInt,
    token: &BytesN<32>,
) -> (BytesN<32>, ContractTest) {
    let id = generate_contract_id();
    register_hello_contract(e, &id);

    let hello_test = ContractTest::new(e, &id);

    // invoked initialize method on the test contract id registered
    hello_test.client().initialize(
        &Identifier::Account(recipient.clone()),
        target_amount,
        token,
    );

    (BytesN::from_array(e, &id), hello_test) // hello_test holds {env,contract_id}
}

// // helper function for mutating the ledger's timestamp in test environment
// fn advance_ledger(e: &Env, delta: u64) {
//     e.ledger().with_mut(|l| {
//         l.timestamp += delta;
//     });
// }

// struct Setup {
//     env: Env,
//     recipient_id: Identifier,
//     user1_id: Identifier,
//     user2: AccountId,
//     user2_id: Identifier,
//     token: Token,
//     contract_test: ContractTest,
//     contract_hello_id: Identifier,
// }

// impl Setup {
//     fn new() -> Self {
//         let e: Env = soroban_sdk::Env::default();

//         let recipient = e.accounts().generate_and_create();
//         let recipient_id = Identifier::Account(recipient.clone());

//         let user1 = e.accounts().generate_and_create();
//         let user1_id = Identifier::Account(user1.clone());

//         let user2 = e.accounts().generate_and_create();
//         let user2_id = Identifier::Account(user2.clone());

//         let target_amount = BigInt::from_i32(&e, 15);

//         let token_admin = e.accounts().generate_and_create();

//         let (contract_token, token) = create_token_contract(&e, &token_admin);

//         let (contract_hello_world, contract_test) =
//             create_hello_contract(&e, &recipient, &target_amount, &contract_token);

//         let contract_hello_id = Identifier::Contract(contract_hello_world);

//         // minting some tokens in soroban environment
//         // minting 10 tokens to user 1
//         token.with_source_account(&token_admin).mint(
//             &Signature::Invoker,
//             &BigInt::zero(&e),
//             &user1_id,
//             &BigInt::from_u32(&e, 10),
//         );

//         // minting 5 tokens to user 2
//         token.with_source_account(&token_admin).mint(
//             &Signature::Invoker,
//             &BigInt::zero(&e),
//             &user2_id,
//             &BigInt::from_u32(&e, 5),
//         );

//         // // approve method
//         // token.with_source_account(&user1).approve(
//         //     &Signature::Invoker,
//         //     &BigInt::zero(&e),
//         //     &contract_hello_id,
//         //     &BigInt::from_u32(&e, 10),
//         // );

//         // Invoking "deposit" method to deposit amount to vault id
//         contract_test.client().deposit();

//         Self {
//             env: e,
//             recipient_id,
//             user1_id,
//             user2,
//             user2_id,
//             token,
//             contract_test,
//             contract_hello_id,
//         }
//     }
// }

fn print_type_of<T>(_: &T) {
    std::println!("Type Of Variable passed is: {}", std::any::type_name::<T>())
}

fn generate_keypair() -> Keypair {
    Keypair::generate(&mut thread_rng())
}

// keypair to Identifier
pub fn to_ed25519(e: &Env, kp: &Keypair) -> Identifier {
    Identifier::Ed25519(kp.public.to_bytes().into_val(e))
}

// find a soln for converting keypair to AccountId ?

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

    // // public and private keys of Vault Account
    // let public_encoded = "GA63NQJB6SXHDVOI3NXP4GM3K5MB4KLTX6R4YK2KKXY4DM27ZNUOVJYY".to_string();
    // let secret_encoded = "SC2ZVG244UNKKBEKAQLEFAS2AU4XGEX5TXCXBTJZ6DXVU5MJ4E4FRKF4".to_string();

    // // Way 01
    // // keypair_identifier and keypair
    // let (vault_account_id, vault_account_kp) = ed25519_utils::build_kp(
    //     &env,
    //     &ed25519_utils::decode_pub(public_encoded),
    //     &ed25519_utils::decode_secret(secret_encoded),
    // );

    // // Way 02
    // let vault_account = env.accounts().generate(); // keypair // `&soroban_sdk::AccountId`
    // let vault_account_id = ?;

    // // Way 03
    // let vault_account = generate_keypair(); // keypair
    // let vault_account_id = to_ed25519(&env, &vault_account); // expects &ed25519_dalek::Keypair` // identifier

    // Way 04
    let (vault_account_id, vault_account_sign) = soroban_auth::testutils::ed25519::generate(&env);

    // this is the target amount; whenever user invokes deposit method; he/she pays 10 token to above vault_account
    let amount: BigInt = BigInt::from_u32(&env, 10);

    // // // generate the token_id randomly
    // // // for fixed token_id
    // // let token_id: BytesN<32> = bytesn!(&env,0x728df6266b618d35c0d924d4e8f5ccbbc1304ad19cf1c6901de672ea849005d5); // added '0x' later // total 66 chars
    // // let token_id: BytesN<32> = generate_contract_id(); // mismatched types: expected struct `soroban_sdk::BytesN<32>`; found array `[u8; 32]`
    // let token_id = &BytesN::from_array(&env, &generate_contract_id());

    // // Identifier::Account(AccountId)
    // let token_admin1 = env.accounts().generate_and_create();
    // std::println!(
    //     "🚀 ~ file: test.rs ~ line 358 ~ fntest2 ~ token_admin1 {:?}",
    //     token_admin1
    // );

    // Identifier::Account(AccountId)
    let token_admin1 = env.accounts().generate_and_create();
    let token_admin1_id = Identifier::Account(token_admin1.clone());

    // Identifier::Ed25519(BytesN<32>)
    let (token_admin_id, token_admin_sign) = soroban_auth::testutils::ed25519::generate(&env);

    // // fn to get AccountId from Identifier::Account(AccountId)
    // fn get_account_id(some_identifier: Identifier) -> Result<AccountId, ()> {
    //     std::println!(
    //         "🚀 ~ file: test.rs ~ line 356 ~ fnget_account_id ~ some_identifier {:?}",
    //         some_identifier,
    //     );
    //     match some_identifier {
    //         Identifier::Account(x) => {
    //             std::println!("found AccountId");
    //             return Ok(x);
    //         }

    //         _ => {
    //             std::println!("error on 381");
    //             return Err(());
    //         }
    //     }
    // }

    // let token_admin = get_account_id(token_admin_id.clone()).unwrap();
    // std::println!(
    //     "🚀 ~ file: test.rs ~ line 358 ~ fntest2 ~ token_admin {:?}",
    //     token_admin
    // );

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
            name: "EBI Token".into_val(&env),
            symbol: "EBI".into_val(&env),
            decimals: 7,
        },
    );

    std::println!("L409 invoked init method of smart contract");

    // let (token_id_bytesn, token_client) = create_token_contract(&env, &token_admin);

    // std::println!("about to use create_token_contract_ed25519");
    // // using create_token_contract_ed25519 // turns out to error: panicked at 'called `Result::unwrap()` on an `Err` value: HostError
    // let (token_id_bytesn, token_client) = create_token_contract_ed25519(&env, &token_admin_id);

    // client.with_source_account(&vault_account).initialize(
    //     &vault_account_id, // &Signature::Invoker, // change this with type identifier
    //     &amount,           //target_amount
    //     &token_id,         // token in bytesn!
    // );

    // // "Initialize method" using .with_source_account(&token_admin)
    // // invoking "initialize" method that sets value in custom data type
    // hello_client.with_source_account(&token_admin).initialize(
    //     &vault_account_id, // &Signature::Invoker, // change this with type identifier
    //     &amount,           //target_amount
    //     &token_id_bytesn,  // token program id in either randomly generated or done by bytesn!
    // );
    // std::println!("initialize method invoked, mutates a state in soroban env");

    // initialize method can be invoked by anyone
    hello_client.initialize(&vault_account_id, &amount, &BytesN::from_array(&env, &id));
    // hello_client.initialize(&vault_account_id, &amount, &token_contract_id);

    // hello_client.with_source_account(&token_admin).initialize(
    //     &vault_account_id, // &Signature::Invoker, // change this with type identifier
    //     &amount,           //target_amount
    //     &token_id_bytesn,  // token program id in either randomly generated or done by bytesn!
    // );
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
        symbol!("testmint"),
        // (&hello_contract_id, &user1_id),
        (&token_admin_id, &user1_id),
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

// seperate test for invoking get_target won't find anything in standalone test == test3
// #[test]
// fn test3() {
//     std::println!("test3 begins");

//     let env = Env::default();
//     let contract_id = env.register_contract(None, Contract); // registering hello world smart contract
//     let client = ContractClient::new(&env, &contract_id); // creating hello world client

//     // // Way 04
//     // let (vault_account_id, vault_account_sign) =
//     //     soroban_auth::testutils::ed25519::generate(&env);

//     // let amount: BigInt = BigInt::from_u32(&env, 50);

//     // let token_id: BytesN<32> =
//     //     bytesn!(&env,0x728df6266b618d35c0d924d4e8f5ccbbc1304ad19cf1c6901de672ea849005d5); // added '0x' later // total 66 chars

//     let assert_data = client.get_target();
//     std::println!("assert_data {:?}", assert_data);

//     assert_eq!(assert_data, BigInt::from_u32(&env, 50));
// }

// #[test]
// fn test_success() {
//     let setup = Setup::new();

//     // setup.token.with_source_account(&setup.user2).approve(
//     //     &Signature::Invoker,
//     //     &BigInt::zero(&setup.env),
//     //     &setup.contract_hello_id,
//     //     &BigInt::from_u32(&setup.env, 5),
//     // );

//     // Invoker deposit amount in vault id
//     setup
//         .contract_test
//         .client()
//         .deposit();

//     std::println!("setup.token.balance(&setup.user1_id): {}", setup.token.balance(&setup.user1_id));
//     std::println!("setup.token.balance(&setup.user2_id): {}", setup.token.balance(&setup.user2_id));
//     std::println!("setup.token.balance(&setup.contract_hello_id): {}", setup.token.balance(&setup.contract_hello_id));
//     std::println!("setup.token.balance(&setup.recipient_id): {}", setup.token.balance(&setup.recipient_id));

//     // assert_eq!(
//     //     setup.token.balance(&setup.user1_id),
//     //     BigInt::zero(&setup.env)
//     // );
//     // assert_eq!(
//     //     setup.token.balance(&setup.user2_id),
//     //     BigInt::zero(&setup.env)
//     // );
//     // assert_eq!(
//     //     setup.token.balance(&setup.contract_hello_id),
//     //     BigInt::from_u32(&setup.env, 20)
//     // );

//     // advance_ledger(&setup.env, 10);

//     // post withdrawal we'll see balance in recipient wallet
//     // setup.crowdfund.client().withdraw(&setup.recipient_id);

//     // assert_eq!(
//     //     setup.token.balance(&setup.user1_id),
//     //     BigInt::zero(&setup.env)
//     // );Identifier
//     // assert_eq!(
//     //     setup.token.balance(&setup.user2_id),
//     //     BigInt::zero(&setup.env)
//     // );
//     // assert_eq!(
//     //     setup.token.balance(&setup.contract_hello_id),
//     //     BigInt::zero(&setup.env)
//     // );
//     // assert_eq!(
//     //     setup.token.balance(&setup.recipient_id),
//     //     BigInt::from_u32(&setup.env, 15)
//     // );
// }
