#![no_std]
use soroban_auth::{Identifier, Signature};
use soroban_sdk::{contractimpl, contracttype, BigInt, BytesN, Env};

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Recipient, // common vault address where everyone will deposit tokens
    Target,    // target amount of tokens that an invoker will transfer to
    TokenId,   // token program id on top of of soroban
}
mod token {
    soroban_sdk::contractimport!(file = "./soroban_token_spec.wasm");
}
mod test_bak;

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
pub struct TestMintContract;

pub trait TestMintContractTrait {
    fn initialize(e: Env, recipient: Identifier, target_amount: BigInt, token: BytesN<32>);
    fn t_mint(e: Env, sig: Signature, amount: BigInt, to: Identifier);
    fn recipient(e: Env) -> Identifier;
    fn target(e: Env) -> BigInt;
    fn tokenid(e: Env) -> BytesN<32>;
}

#[contractimpl]
impl TestMintContractTrait for TestMintContract {
    // this method only setup data in custom data type created
    // to be invoked only once
    fn initialize(e: Env, recipient: Identifier, target_amount: BigInt, token: BytesN<32>) {
        assert!(!e.data().has(DataKey::Recipient), "already initialized");
        e.data().set(DataKey::Recipient, recipient);
        e.data().set(DataKey::Target, target_amount);
        e.data().set(DataKey::TokenId, token);
    }

    fn recipient(e: Env) -> Identifier {
        get_recipient(&e)
    }

    fn target(e: Env) -> BigInt {
        get_target_amount(&e)
    }

    fn tokenid(e: Env) -> BytesN<32> {
        get_token_id(&e)
    }

    fn t_mint(e: Env, sig: Signature, amount: BigInt, to: Identifier) {
        let tok = get_token_id(&e);
        let client = token::Client::new(&e, tok);
        client.mint(&sig, &client.nonce(&sig.identifier(&e)), &to, &amount);
    }
}
