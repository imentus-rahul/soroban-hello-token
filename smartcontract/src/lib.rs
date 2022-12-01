#![no_std]
use soroban_auth::{Identifier, Signature};
use soroban_sdk::{
    contractimpl, contracttype, log, symbol, vec, AccountId, BigInt, Bytes, BytesN, Env, Symbol,
    Vec,
};

mod token {
    soroban_sdk::contractimport!(file = "./soroban_token_spec.wasm");
}

mod test;
mod testutils;

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Recipient, // common vault address where everyone will deposit tokens
    Target,    // target amount of tokens that an invoker will transfer to
    TokenId,   // token program id on top of of soroban
}

// helper functions
// pass token contract id in this helper function
fn transfer_token_soroban(e: &Env, contract_id: &BytesN<32>, to: &Identifier, amount: &BigInt) {
    let nonce: BigInt = BigInt::zero(e);
    let client = token::Client::new(e, contract_id); // cross program invocation // by creating client
    client.xfer(&Signature::Invoker, &nonce, to, amount); // from == invoker
}

fn get_contract_id(e: &Env) -> Identifier {
    Identifier::Contract(e.get_current_contract())
}

// fn get_token_contract_id(e: &Env) -> Identifier {
//     Identifier::Contract(get_token_id(e))
// }

// getter functions for custom data type // setters are directly called in "initialize" method

fn get_recipient(e: &Env) -> Identifier {
    let key = DataKey::Recipient;
    e.data().get_unchecked(key).unwrap()
}

fn get_target_amount(e: &Env) -> BigInt {
    let key = DataKey::Target;
    e.data().get_unchecked(key).unwrap()
}

fn get_token_id(e: &Env) -> BytesN<32> {
    let key = DataKey::TokenId;
    // e.data().get(key).unwrap().unwrap()
    e.data().get_unchecked(key).unwrap()
}

// pass token contract id in this helper function
fn get_token_balance_soroban(e: &Env, token_contract_id: &BytesN<32>, account_id:&Identifier) -> BigInt {
    token::Client::new(&e, token_contract_id).balance(account_id)
}

pub struct Contract;

#[contractimpl]
impl Contract {
    // trying invocation with mutiple parameters
    pub fn hello(env: Env, to: Symbol, param2: u64) -> Vec<Symbol> {
        log!(&env, "fn hello params: to: {} param2: {}", to, param2);
        vec![&env, symbol!("Hello"), to] // not passing param2 in output
    }

    pub fn world(env: Env) -> Vec<Symbol> {
        log!(&env, "fn world");
        vec![&env, symbol!("Hello")]
    }

    // this method only setup data in custom data type created
    // to be invoked only once
    pub fn initialize(e: Env, recipient: Identifier, target_amount: BigInt, token: BytesN<32>) {
        // assert!(!e.data().has(DataKey::Recipient), "already initialized");
        e.data().set(DataKey::Recipient, recipient);
        e.data().set(DataKey::Target, target_amount);
        e.data().set(DataKey::TokenId, token);
    }

    pub fn recipient(e: Env) -> Identifier {
        get_recipient(&e)
    }

    pub fn target(e: Env) -> BigInt {
        get_target_amount(&e)
    }

    pub fn tokenid(e: Env) -> BytesN<32> {
        get_token_id(&e)
    }

    pub fn balance(e: Env, account_id:Identifier) -> BigInt {
        get_token_balance_soroban(&e, &get_token_id(&e), &account_id)
    }

    // next method is to deposit the classic stellar token from invoker's account to vault account
    pub fn deposit(e: Env, sig: Signature) {
        let client = token::Client::new(&e, get_token_id(&e)); // token client
        let nonce = BigInt::zero(&e);
        // let nonce = client.nonce(&Signature::Invoker.identifier(&e)); // this may not work

        // token fn declarations
        // fn xfer(e: Env, from: Signature, nonce: BigInt, to: Identifier, amount: BigInt);
        // fn xfer_from(e: Env, spender: Signature, nonce: BigInt, from: Identifier, to: Identifier, amount: BigInt);

        // client.xfer_from(
        //     &Signature::Invoker,    //spender - signature
        //     &nonce,                // nonce
        //     &user,                 // from - identifier
        //     &get_contract_id(&e),   // to -identifier
        //     &amount,                // amount - BigInt
        // );

        let vault_id = get_recipient(&e); // set recipient that was already set in DataKey::Recipient
        let target_amount = get_target_amount(&e); // get target amount that a user needs to transfer
        // client.xfer(&Signature::Invoker, &nonce, &vault_id, &target_amount);
        client.xfer(&sig, &nonce, &vault_id, &target_amount);
    }

    // next method is to withdraw the classic stellar token from vault account to invoker's account
    pub fn withdraw(e: Env, user_id: Identifier) {
        let client = token::Client::new(&e, get_token_id(&e));
        let nonce = BigInt::zero(&e);

        // token fn declarations
        // fn xfer(e: Env, from: Signature, nonce: BigInt, to: Identifier, amount: BigInt);
        // fn xfer_from(e: Env, spender: Signature, nonce: BigInt, from: Identifier, to: Identifier, amount: BigInt);

        // client.xfer_from(
        //     &Signature::Invoker,    //spender - signature
        //     &nonce,                // nonce
        //     &user,                 // from - identifier
        //     &get_contract_id(&e),   // to -identifier
        //     &amount,                // amount - BigInt
        // );

        let vault_id = get_recipient(&e); // set recipient from DataKey::Recipient
        let target_amount = get_target_amount(&e);

        // Expected transfer is from vault account to the account passed in withdraw method
        // therefore this account would be invoked by vault account private key
        // client.xfer(SignatureOfVaultAccount, &nonce, &INVOKER_IDENTIFIER, &target_amount);
        client.xfer(&Signature::Invoker, &nonce, &user_id, &target_amount);
    }

    // test method mint
    pub fn testmint(e: Env, sig: Signature, to: Identifier) {

        let id = sig.identifier(&e);
        log!(&e, "SC: id: {} ", &id);


        let invoker = e.invoker();
        log!(&e, "SC: Invoker in mint method - here the invoker should be the smart contract: {} ", invoker);

        // token_admin: AccountId
        let client = token::Client::new(&e, get_token_id(&e));
        let nonce = BigInt::zero(&e);
        // let nonce = client.nonce(&Signature::Invoker.identifier(&e)); // this may not work
        // let nonce = client.nonce(&sig.identifier(&e)); // this may not work
        log!(&e, "SC: nonce: {} ", &nonce);


        let target_amount = get_target_amount(&e); // get target amount that a user needs to transfer
        log!(&e, "SC: target_amount: {} ", &target_amount);

        let target_amount1 = BigInt::from_u32(&e, 100);
        log!(&e, "SC: target_amount1: {} ", &target_amount1);

        // client.mint(&Signature::Invoker, &nonce, &to, &target_amount); // this won't work as invoker is smart contract
        client.mint(&sig, &nonce, &to, &target_amount);
    }

    // test method name
    pub fn name(e: Env) -> Bytes {
        let invoker = e.invoker();
        log!(&e, "SC: Invoker in name method: {} ", invoker);

        let client = token::Client::new(&e, get_token_id(&e));
        client.name()
    }
}
