#![cfg_attr(not(feature = "std"), no_std)]

use ink_env::Environment;
use ink_lang as ink;
use ink_prelude::{ vec::Vec, format };

/// Define the operations to interact with the substrate runtime
#[ink::chain_extension]
pub trait FetchRandom {
    type ErrorCode = RandomReadErr;

    /// Note: this gives the operation a corresponding func_id (1101 in this case),
    /// and the chain-side chain_extension will get the func_id to do further operations.
    #[ink(extension = 1101, returns_result = false)]
    fn fetch_random() -> [u8; 32];

    #[ink(extension = 1102, returns_result = false)]
    fn create_claim(claim: Vec<u8>);

    #[ink(extension = 1103, returns_result = false)]
    fn create_kitty() -> u32;
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum RandomReadErr {
    FailGetRandomSource,
}

impl ink_env::chain_extension::FromStatusCode for RandomReadErr {
    fn from_status_code(status_code: u32) -> Result<(), Self> {
        match status_code {
            0 => Ok(()),
            1 => Err(Self::FailGetRandomSource),
            _ => panic!("encountered unknown status code"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum CustomEnvironment {}

impl Environment for CustomEnvironment {
    const MAX_EVENT_TOPICS: usize =
        <ink_env::DefaultEnvironment as Environment>::MAX_EVENT_TOPICS;

    type AccountId = <ink_env::DefaultEnvironment as Environment>::AccountId;
    type Balance = <ink_env::DefaultEnvironment as Environment>::Balance;
    type Hash = <ink_env::DefaultEnvironment as Environment>::Hash;
    type BlockNumber = <ink_env::DefaultEnvironment as Environment>::BlockNumber;
    type Timestamp = <ink_env::DefaultEnvironment as Environment>::Timestamp;

    type ChainExtension = FetchRandom;
}

#[ink::contract(env = crate::CustomEnvironment)]

mod randkey {
    use super::RandomReadErr;
    use crate::{Vec, format};

    /// Defines the storage of your contract.
    /// Add new fields to the below struct in order
    /// to add new static storage fields to your contract.
    #[ink(storage)]
    pub struct Randkey {
        /// Stores a single `bool` value on the storage.
        value: [u8; 32],
        kitty_id: u32,
    }
    #[ink(event)]
    pub struct RandomUpdated{
        #[ink(topic)]
        new: [u8; 32],
    }
    #[ink(event)]
    pub struct ClaimCreated{
        #[ink(topic)]
        claim: Vec<u8>,
    }
    #[ink(event)]
    pub struct KittyCreated{
        #[ink(topic)]
        kitty_id: u32,
    }

    impl Randkey {
        /// Constructor that initializes the `bool` value to the given `init_value`.
        #[ink(constructor)]
        pub fn new(init_value: [u8; 32]) -> Self {
            Self { value: init_value, kitty_id: Default::default() }
        }

        /// Constructor that initializes the `bool` value to `false`.
        ///
        /// Constructors can delegate to other constructors.
        #[ink(constructor)]
        pub fn default() -> Self {
            Self::new( Default::default() )
        }

        /// A message that can be called on instantiated contracts.
        /// This one flips the value of the stored `bool` from `true`
        /// to `false` and vice versa.
        #[ink(message)]
        pub fn update(&mut self) -> Result<(), RandomReadErr> {
            let new_randomkey = self.env().extension().fetch_random()?;
            self.value = new_randomkey;

            let message = format!("randdomkey =  {:?}", new_randomkey);
            ink_env::debug_println(&message);
            
            self.env().emit_event(RandomUpdated{ new: new_randomkey });

            Ok(())
        }

        /// Call Claim Created 
        #[ink(message)]
        pub fn create_claim(&mut self, claim: Vec<u8>) -> Result<(), RandomReadErr> {
            self.env().extension().create_claim( claim.clone() )?;

            self.env().emit_event(ClaimCreated{ claim: claim });
            Ok(())
        }


        #[ink(message)]
        pub fn create_kitty(&mut self) -> Result<(), RandomReadErr> {

            let id = self.env().extension().create_kitty()?;
            
            let message = format!("kitty id =  {:?}", id);
            ink_env::debug_println(&message);

            self.env().emit_event(KittyCreated{ kitty_id: id.clone() });

            let message = format!("Emit_event");
            ink_env::debug_println(&message);

            self.kitty_id = id;

            Ok(())
        }

        /// Simply returns the current value of our `bool`.
        #[ink(message)]
        pub fn get(&self) -> [u8; 32] {
            self.value
        }

        #[ink(message)]
        pub fn get_kitty_id(&self) -> u32 {

            self.kitty_id.clone()
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
            let randkey = Randkey::default();
            assert_eq!(randkey.get(), false);
        }

        /// We test a simple use case of our contract.
        #[test]
        fn it_works() {
            let randkey = RandExtension::default();
            assert_eq!(randkey.get(), [0; 32]);
        }
    }
}
