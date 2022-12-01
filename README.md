## PoC on Cross Program Invocation (CPI) in Soroban
- idea is to create a smart contract "hello_world" that invokes "mint" and "xfer" method of "token smart contract.
- there is a "testmint" method in "hello world" that should mint tokens to whosoever calls that method.
- In the first phase we are creating tokens in soroban itself rather than wrapping a classical asset.

### Blocker Dtd: 25/11/22

- Blocker: At L199 https://github.com/imentus-rahul/soroban-hello-token/blob/main/smartcontract/src/test.rs#L199, I'm trying to make a cross-program invocation, but it doesn't work. 
- the idea is to invoke the "mint" method of "token smart contract" from "hello world smart contract" 
- for doing same, created a "testmint" method in "hello world smart contract" 

- I tried invoking "mint"/"xfer" method directly in client using token_client here https://github.com/imentus-rahul/soroban-hello-token/blob/main/smartcontract/src/test.rs#L153 and it works well. 

- Though when trying to do same using "testmint"/"deposit" method in "hello world smart contract", turns to an error. 

- To replicate error: clone the repo and run `cargo test --package soroban-hello-token --lib -- test --nocapture` from "smartcontract" folder

- Comment Error Line to run through all test cases: https://github.com/imentus-rahul/soroban-hello-token/blob/main/smartcontract/src/test.rs#L199

- Applied Solution from community: https://soroban.stellar.org/docs/examples/auth-advanced#testing-auth-by-ed25519

- I know I'm creating signatures not in correct manner, but what's the correct way then?

  - Still stuck with errors like:
  - NativeContract::call at Line 199 and 638
'''
thread 'test::test2' panicked at 'called`Result::unwrap()`on an`Err` value: HostError
    Value: Status(UnknownError(0))
Backtrace (newest first):
0: <soroban_env_host::native_contract::token::contract::Token as soroban_env_host::native_contract::NativeContract>::call
at /home/imentus/.cargo/registry/src/github.com-1ecc6299db9ec823/soroban-env-host-0.0.6/src/native_contract/token/contract.rs:112:1
'''

- In talks with the community for the solution here: https://discord.com/channels/897514728459468821/1046693199130927104