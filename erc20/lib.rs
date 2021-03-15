#![cfg_attr(not(feature = "std"), no_std)]

pub use self::erc20::{Error, Erc20, Result};
use ink_lang as ink;

#[ink::contract]
pub mod erc20 {
    use ink_storage::collections::HashMap as StorageHashMap;
    use ink_prelude::vec::Vec;

    // 定义数据存储，参考 ERC20 标准，不过根据 RUST 的规范，将驼峰式修改为下划线命名法
    #[ink(storage)]
    pub struct Erc20 {
        creater: AccountId,
        // 代币名称
        name: Vec<u8>,
        // 代币标识
        symbol: Vec<u8>,
        // 定义代币供应总量
        total_supply:Balance,
        // 存储各个账号的余额
        balances : StorageHashMap<AccountId, Balance>,
        // 授权某人可以使用自己的余额
        allowances : StorageHashMap<(AccountId, AccountId), Balance>,
    }

    // 定义事件，ink(topic) 标识有需要通过这个字段查询时间的需求
    // 因为发行或者销毁的时候，from 或者 to 会是 none ，所以需要转账的 from 和 to 设置为 Option
    // 因为授权，两个账号必须存在，所以 owner 和 spender 不需要 Option
    #[ink(event)]
    pub struct Transfer {
        #[ink(topic)]
        from: Option<AccountId>,
        #[ink(topic)]
        to: Option<AccountId>,
        #[ink(topic)]
        value: Balance,
    }

    #[ink(event)]
    pub struct Approval {
        #[ink(topic)]
        owner: AccountId,
        #[ink(topic)]
        spender: AccountId,
        #[ink(topic)]
        value: Balance,
    }

    // 定义不同错误的的枚举类型，
    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        InsufficientBalance,
        InsufficientAllowance,
        OnlyForCreater,
    }

    // 定义返回类型，当有返回值也可能返回错误的函数，需要用 Result 类型返回
    pub type Result<T> = core::result::Result<T, Error>;

    impl Erc20 {
        // 初始化部署代币
        // name : 代币名称，如 BitCoin
        // symbol : 代币标识，如 BTC
        // total_subbly : 总供应量
        #[ink(constructor)]
        pub fn new(name: Vec<u8>, symbol: Vec<u8>, total_supply: Balance) -> Self {
            // 获取部署的调用者
            let caller = Self::env().caller();
            // 定义余额数据，将所有发行的代币，都放给部署账号
            let mut balances = StorageHashMap::new();
            balances.insert(caller, total_supply);
            // 定义数据存储
            let instance = Self {
                creater : caller,
                name: name,
                symbol: symbol,
                total_supply: total_supply,
                balances: balances,
                allowances: StorageHashMap::new()
            };
            // 触发转账事件，因为第一笔发行，也是一种转账
            Self::env().emit_event(Transfer {
                from: None,
                to: Some(caller),
                value: total_supply,
            });

            instance
        }

        // 返回代币名称，如 BitCoin
        #[ink(message)]
        pub fn name(&self) -> Vec<u8>{
            self.name.clone()
        }

        // 返回代币标识，比如 BTC
        #[ink(message)]
        pub fn symbol(&self) -> Vec<u8>{
            self.symbol.clone()
        }

        // 返回代币总供应量
        #[ink(message)]
        pub fn total_supply(&self) -> Balance{
            self.total_supply
        }
        
        // 返回指定账号的余额
        #[ink(message)]
        pub fn balance_of(&self, of: AccountId) -> Balance{
            // 返回的值是 &Balance 的类型，所以需要 * 解引用
            // 可以使用 copied ，这样就不需要解引用了
            let balance = self.balances.get(&of).unwrap_or(&0);
            *balance
        }
        
        // // 向指定账号转账
        #[ink(message)]
        pub fn transfer(&mut self, to: AccountId, value:Balance) -> Result<()>{
            // 获取调用者
            let caller = Self::env().caller();

            self.transfer_from_to(Some(caller), Some(to), value)
        }

        // 授权某账号可以使用自己的账户余额
        #[ink(message)]
        pub fn approve(&mut self, spender: AccountId, value: Balance) -> Result<()>{
            let caller = Self::env().caller();
            // 插入授权的记录，授权是未来花费，所以不需要考虑当前是否有余额是否足够，
            self.allowances.insert((caller, spender), value);

            self.env().emit_event( Approval{
                owner : caller,
                spender : spender,
                value : value,
            });
            Ok(())
        }

        // 获取第一个账户授权第二个账户可使用的数量
        #[ink(message)]
        pub fn allowance(&self, owner: AccountId, spender: AccountId) -> Balance {
            // 或者 self.allowances.get(&(owner, spender)).copied().unwrap_or(0)
            // 因为 copied 后，就不需要解引用(*)了
            *self.allowances.get(&(owner, spender)).unwrap_or(&0)
        }
        
        // 在授权(allowance)范围内，将指定账号的代币转到指定账号
        #[ink(message)]
        pub fn transfer_from(&mut self, from: AccountId, to: AccountId, value: Balance) -> Result<()>{
            let caller = Self::env().caller();
            let allowance = self.allowance(from, caller);
            if allowance < value {
                return Err(Error::InsufficientAllowance)
            }
            self.transfer_from_to(Some(from), Some(to) , value)?;

            self.allowances.insert((from, to), allowance - value);
            
            Ok(())
        }

        // 内部函数，用于从一个账户转账到另外一个账户
        fn transfer_from_to(&mut self, from: Option<AccountId>, to: Option<AccountId>, value:Balance) -> Result<()>{
            // 判断 from 账户是否有足够多的钱
            if let Some(from_account) = from {
                let from_balance = self.balance_of(from_account);
                if from_balance < value {
                    return Err(Error::InsufficientBalance)
                }
                self.balances.insert(from_account, from_balance - value);
            }
            if let Some(to_account) = to {
                let to_balance = self.balance_of(to_account);
                self.balances.insert(to_account, to_balance + value);
            }
            
            self.env().emit_event( Transfer{
                from : from,
                to : to,
                value : value
            });
            Ok(())
        }

        // 增发代币，只能创建者可以增发，增发的会直接转账给创建者，增发需要增加总供应量
        #[ink(message)]
        pub fn issue(&mut self, amount: Balance) -> Result<()>{
            let caller = Self::env().caller();
            if caller != self.creater {
                return Err(Error::OnlyForCreater)
            }
            let total_supply = self.total_supply();
            self.total_supply = total_supply + amount;

            self.transfer_from_to(None, Some(caller) , amount)?;
            Ok(())
        }

        // 销毁代币，任何账号都可以销毁自己持有的代币，销毁后需要减少总供应量
        #[ink(message)]
        pub fn burn(&mut self, amount: Balance) -> Result<()>{
            let caller = Self::env().caller();
            self.transfer_from_to(Some(caller), None, amount)?;
            let total_supply = self.total_supply();
            self.total_supply = total_supply - amount;

            Ok(())
        }
    }

    /// Unit tests in Rust are normally defined within such a `#[cfg(test)]`
    /// module and test functions are marked with a `#[test]` attribute.
    /// The below code is technically just normal Rust code.
    #[cfg(test)]
    mod tests {

        use super::*;
        use ink_env::{
            hash::{
                Blake2x256,
                CryptoHash,
                HashOutput,
            },
            Clear,
        };

        use ink_lang as ink;

        type Event = <Erc20 as ::ink_lang::BaseEvent>::Type;

        // 检测转账事件是否匹配
        fn assert_transfer_event(
            event: &ink_env::test::EmittedEvent,
            expected_from: Option<AccountId>,
            expected_to: Option<AccountId>,
            expected_value: Balance,
        ) {

            // 将事件解码
            let decoded_event = <Event as scale::Decode>::decode(&mut &event.data[..])
                .expect("encountered invalid contract event data buffer");
            
            // 如果匹配解码数据格式是转账事件，并且确认事件的各个值是否匹配
            if let Event::Transfer(Transfer { from, to, value }) = decoded_event {
                assert_eq!(from, expected_from, "encountered invalid Transfer.from");
                assert_eq!(to, expected_to, "encountered invalid Transfer.to");
                assert_eq!(value, expected_value, "encountered invalid Trasfer.value");
            } else {
                panic!("encountered unexpected event kind: expected a Transfer event")
            }
            fn encoded_into_hash<T>(entity: &T) -> Hash
            where
                T: scale::Encode,
            {
                let mut result = Hash::clear();
                let len_result = result.as_ref().len();
                let encoded = entity.encode();
                let len_encoded = encoded.len();
                if len_encoded <= len_result {
                    result.as_mut()[..len_encoded].copy_from_slice(&encoded);
                    return result
                }
                let mut hash_output =
                    <<Blake2x256 as HashOutput>::Type as Default>::default();
                <Blake2x256 as CryptoHash>::hash(&encoded, &mut hash_output);
                let copy_len = core::cmp::min(hash_output.len(), len_result);
                result.as_mut()[0..copy_len].copy_from_slice(&hash_output[0..copy_len]);
                result
            }
            let expected_topics = vec![
                encoded_into_hash(&PrefixedValue {
                    value: b"Erc20::Transfer",
                    prefix: b"",
                }),
                encoded_into_hash(&PrefixedValue {
                    prefix: b"Erc20::Transfer::from",
                    value: &expected_from,
                }),
                encoded_into_hash(&PrefixedValue {
                    prefix: b"Erc20::Transfer::to",
                    value: &expected_to,
                }),
                encoded_into_hash(&PrefixedValue {
                    prefix: b"Erc20::Transfer::value",
                    value: &expected_value,
                }),
            ];
            for (n, (actual_topic, expected_topic)) in
                event.topics.iter().zip(expected_topics).enumerate()
            {
                let topic = actual_topic
                    .decode::<Hash>()
                    .expect("encountered invalid topic encoding");
                assert_eq!(topic, expected_topic, "encountered invalid topic at {}", n);
            }
        }

        //  测试创建合约
        #[ink::test]
        fn create_works() {
            let erc20 = Erc20::new(b"xDOT".to_vec(), b"DOT".to_vec(), 1_000_000_000);
            // 检查创建的是各项属性是否设置正确
            assert_eq!(erc20.name(), b"xDOT".to_vec());
            assert_eq!(erc20.symbol(), b"DOT".to_vec());
            assert_eq!(erc20.total_supply(), 1_000_000_000);
            assert_eq!(erc20.balance_of(AccountId::from([0x01; 32])), 1_000_000_000);

            // 检测触发的时间是不是1个
            let emitted_events = ink_env::test::recorded_events().collect::<Vec<_>>();
            assert_eq!(1, emitted_events.len());

            assert_transfer_event(
                &emitted_events[0],
                None,
                Some(AccountId::from([0x01; 32])),
                1_000_000_000,
            );
        }
        #[ink::test]
        fn transfer_works() {
            // 后边会需要调用修改的接口，所以需要加 mut
            let mut erc20 = Erc20::new(b"xDOT".to_vec(), b"DOT".to_vec(), 1_000_000_000);
            // 返回用于测试的账号(Alice, Bob, Charlie, Django, Eve , Frank)
            let accounts =
                ink_env::test::default_accounts::<ink_env::DefaultEnvironment>()
                    .expect("Cannot get accounts");

            // bob 的余额为0
            assert_eq!(erc20.balance_of(accounts.bob), 0);
            // Alice 转账 10 枚代币给 Bob.
            assert_eq!(erc20.transfer(accounts.bob, 10), Ok(()));
            // Bob 余额应该是 10.
            assert_eq!(erc20.balance_of(accounts.bob), 10);

            // 获取事件
            let emitted_events = ink_env::test::recorded_events().collect::<Vec<_>>();
            // 应该有两个事件，一个是创建代币转账给 Alice 的，一个是 Alice 转账给 Bob 的
            assert_eq!( emitted_events.len(), 2);

            // 检测第一个事件创建时候直接转给 Alice 的
            assert_transfer_event(
                &emitted_events[0],
                None,
                Some(AccountId::from([0x01; 32])),
                1_000_000_000,
            );
            // 检测第二个，是由 Alice 转给 Bob 的
            assert_transfer_event(
                &emitted_events[1],
                Some(AccountId::from([0x01; 32])),
                Some(AccountId::from([0x02; 32])),
                10,
            );
        }

        #[ink::test]
        fn transfer_from_works(){
            // 后边会需要调用修改的接口，所以需要加 mut
            let mut erc20 = Erc20::new(b"xDOT".to_vec(), b"DOT".to_vec(), 1_000_000_000);
            // 返回用于测试的账号(Alice, Bob, Charlie, Django, Eve , Frank)
            let accounts =
                ink_env::test::default_accounts::<ink_env::DefaultEnvironment>()
                    .expect("Cannot get accounts");

            // 从 alice 往 eve 账号转账会报错，错误是没有授权
            assert_eq!(
                erc20.transfer_from(accounts.alice, accounts.eve, 10),
                Err(Error::InsufficientAllowance)
            );
            // 授权 Bob 可以代表自己转账 10 个代币
            assert_eq!(erc20.approve(accounts.bob, 10), Ok(()));

            // 应该有两个事件发生，部署时候的初始转账，授权的事件
            assert_eq!(ink_env::test::recorded_events().count(), 2);

            // 获得当前合约部署的地址
            let callee = ink_env::account_id::<ink_env::DefaultEnvironment>()
                .unwrap_or([0x0; 32].into());
            // Create call.
            // 创建 call 
            let mut data =
                ink_env::test::CallData::new(ink_env::call::Selector::new([0x00; 4]));
            data.push_arg(&accounts.bob);
            // 将 Bob 设置为调用者
            ink_env::test::push_execution_context::<ink_env::DefaultEnvironment>(
                accounts.bob,
                callee,
                1000000,
                1000000,
                data,
            );

            // Bob 调用，从 Alice 账号转 10 个代币给 Eve
            assert_eq!(
                erc20.transfer_from(accounts.alice, accounts.eve, 10),
                Ok(())
            );
            // 确认 Eve 的代币数量
            assert_eq!(erc20.balance_of(accounts.eve), 10);

            // 检查所有的转账事件（第一个和第三个，第二个是授权的，跳过）
            let emitted_events = ink_env::test::recorded_events().collect::<Vec<_>>();
            assert_eq!(emitted_events.len(), 3);
            assert_transfer_event(
                &emitted_events[0],
                None,
                Some(AccountId::from([0x01; 32])),
                1_000_000_000,
            );
            assert_transfer_event(
                &emitted_events[2],
                Some(AccountId::from([0x01; 32])),
                Some(AccountId::from([0x05; 32])),
                10,
            );

        }


    }
    /// For calculating the event topic hash.
    struct PrefixedValue<'a, 'b, T> {
        pub prefix: &'a [u8],
        pub value: &'b T,
    }

    impl<X> scale::Encode for PrefixedValue<'_, '_, X>
    where
        X: scale::Encode,
    {
        #[inline]
        fn size_hint(&self) -> usize {
            self.prefix.size_hint() + self.value.size_hint()
        }

        #[inline]
        fn encode_to<T: scale::Output + ?Sized>(&self, dest: &mut T) {
            self.prefix.encode_to(dest);
            self.value.encode_to(dest);
        }
    }
}
