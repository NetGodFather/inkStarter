#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

use ink_prelude::format;

#[ink::contract]
mod loan {
    use ink_storage::collections::HashMap as StorageHashMap;
    use erc20::Erc20;
    use ink_env::call::FromAccountId;
    use crate::format;

    #[ink(storage)]
    pub struct Loan {
        // 合约管理者
        owner: AccountId,
        // 解除币种的合约地址
        base_token_accountid : AccountId,
        // 剩余可借出数量
        borrowings_balance : Balance,
        // 总共借出的数量
        total_borrowings : Balance,
        // 最大借款比例 质押币种 -> 借款比例
        min_collateral_ratio: StorageHashMap< AccountId, u32>,
        // 质押代币数据，(用户, 质押币种) -> 质押数量
        pledges : StorageHashMap<(AccountId, AccountId), Balance>,
        // 借款数量：借款用户 -> 借款数量
        borrowings : StorageHashMap<AccountId, Balance>,
    }
    // 定义返回类型，当有返回值也可能返回错误的函数，需要用 Result 类型返回
    pub type Result<T> = core::result::Result<T, Error>;

    // 定义不同错误的的枚举类型，
    #[derive(Debug, PartialEq, Eq, scale::Encode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        OnlyForOwner,
    }

    impl Loan {
        /// Constructor that initializes the `bool` value to the given `init_value`.
        #[ink(constructor)]
        pub fn new(token: AccountId) -> Self {
            let caller = Self::env().caller();
            Self {
                owner: caller,
                base_token_accountid: token,
                borrowings_balance: 0,
                total_borrowings: 0,
                min_collateral_ratio: StorageHashMap::new(),
                pledges: StorageHashMap::new(),
                borrowings: StorageHashMap::new(),
            }
        }


        #[ink(message)]
        pub fn borrowings_balance( &self ) -> Balance{
            self.borrowings_balance
        }

        #[ink(message)]
        pub fn total_borrowings( &self ) -> Balance{
            self.total_borrowings
        }

        // Rechage base token for borrowing
        #[ink(message)]
        pub fn recharge_for_borrowing(&mut self, amount: Balance) -> Result<()> {
            let caller = Self::env().caller();
            if caller != self.owner {
                return Err(Error::OnlyForOwner)
            }
            let mut base_token: Erc20 = FromAccountId::from_account_id( self.base_token_accountid );

            let self_accountid = Self::env().account_id();

            let message = format!("self_accountid =  {:?}", self_accountid);
            ink_env::debug_println(&message);

            let re = base_token.transfer_from( caller, self_accountid, amount);

            let message = format!("Return =  {:?}", re);
            ink_env::debug_println(&message);


            self.borrowings_balance = self.borrowings_balance + amount;
            
            Ok(())
        }

    }

    /// Unit tests in Rust are normally defined within such a `#[cfg(test)]`
    /// module and test functions are marked with a `#[test]` attribute.
    /// The below code is technically just normal Rust code.
    #[cfg(test)]
    mod tests {
        /// Imports all the definitions from the outer scope so we can use them here.
        use super::*;

        /// We test if the default constructor does its job.
        #[test]
        fn default_works() {
            let loan = Loan::new();
            assert_eq!()

        }

        /// We test a simple use case of our contract.
        #[test]
        fn it_works() {
            let mut loan = Loan::new(false);
            assert_eq!(loan.get(), false);
            loan.flip();
            assert_eq!(loan.get(), true);
        }
    }
}
