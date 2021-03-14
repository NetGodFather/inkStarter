#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract]
mod delegate {
    use erc20::{
        Erc20,
        StandardToken,
    };
    use ink_env::call::FromAccountId;

    #[ink(storage)]
    pub struct Delegate {
        token: StandardToken,
    }

    impl Delegate {
        #[ink(constructor)]
        pub fn new(contract_account: AccountId) -> Self {
            let token: StandardToken = FromAccountId::from_account_id(contract_account);
            Self { token }
        }

        #[ink(message)]
        pub fn call(&self, owner: AccountId) -> Balance {
            self.token.balance_of(owner)
        }
    }
}