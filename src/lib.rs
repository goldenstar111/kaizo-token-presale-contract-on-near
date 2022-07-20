use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{ LookupMap};
use near_sdk::json_types::{ U128 };
use near_sdk::{
    env, near_bindgen, ext_contract, AccountId, PanicOnDefault, Promise, Timestamp, Gas, BorshStorageKey, Balance
};

near_sdk::setup_alloc!();

pub type TimestampSec = u64;
pub type TokenId = String;

const GAS_FOR_FT_TRANSFER: Gas = 5_000_000_000_000;

/// external contract calls

#[ext_contract(ext_fungible_token)]
pub trait FungibleToken {
    fn ft_transfer(&mut self, receiver_id: AccountId, amount: U128, memo: Option<String>);
}


#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    //contract owner
    pub owner_id: AccountId,
	
	pub treasury_id: AccountId,
    //keeps track of all the token IDs for a given account
    pub user_info: LookupMap<AccountId, u64>,
	
	pub ft_token_id: TokenId,

    pub token_price: Balance,
	
	pub start_time: u64,
	
	pub end_time: u64,
	
	pub current_sale: u64,
	
	pub total_sale: u64,
}

/// Helper structure for keys of the persistent collections.
#[derive(BorshStorageKey,BorshSerialize)]
pub enum StorageKey {
    UserInfo,
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new(owner_id: AccountId) -> Self {
        //create a variable of type Self with all the fields initialized. 
        Self {
            owner_id: owner_id.into(),
			treasury_id: "bd286c2c61fc6633b4866f8ddc31a838cfea24042a6c281f1180cc21ed7fbfae".to_string(),
			user_info: LookupMap::new(StorageKey::UserInfo),
			ft_token_id: "dojos.near".to_string(),
			token_price: 3000000000000000000,
			start_time: 1647007905,
			end_time: 1647607905,
			current_sale: 0,
			total_sale: 1000000,
        }
    }
	
	pub fn set_ft_contract(&mut self, contract_id: AccountId) {
        self.assert_owner();
        self.ft_token_id = contract_id;
    }
	
	pub fn set_owner(&mut self, owner_id: AccountId) {
        self.assert_owner();
        self.owner_id = owner_id;
    }
	
	pub fn set_treasiry_id(&mut self, owner_id: AccountId) {
        self.assert_owner();
        self.treasury_id = owner_id;
    }
	
	pub fn get_status(&self ) -> Vec<u64>{
         let vec = vec![self.current_sale, self.total_sale];
		 return vec;
    }
	
	pub fn get_amount_by_owner(&self, account_id: AccountId ) -> u64{
		let tokens = self.user_info.get(&account_id);
		if let Some(tokens) = tokens {
			tokens
		} else {
			0
		}
    }
	
	pub fn get_locked_period(&self) -> Vec<TimestampSec>{
		vec![self.start_time, self.end_time] 
    }
	
	pub fn set_start_time(&mut self, start_time: u64) {
        self.assert_owner();
        self.start_time = start_time;
    }
	
	pub fn set_end_time(&mut self, end_time: u64) {
        self.assert_owner();
        self.end_time = end_time;
    }
	
	pub fn get_token_price(&self) -> Balance {
        self.token_price
    }
	
	pub fn set_token_price(&mut self, price: U128) {
        self.assert_owner();
        self.token_price = price.0;
    }
	
	#[payable]
	pub fn buy(&mut self, account_id: AccountId, token_amount: u64) {
		let require_amount = token_amount as u128 * self.token_price ;
		let attached_deposit = env::attached_deposit();
		let current_stimetamp = self.to_sec(env::block_timestamp());
		assert!(
            self.start_time >= current_stimetamp,
            "Sale not started."
        );
		
		assert!(
            attached_deposit >= require_amount.into(),
            "Not enough attached deposit to buy"
        );
		let initial_storage_usage = env::storage_usage();
		
        let mut totals = self.user_info.get(&account_id).unwrap_or(0);
		totals += token_amount;
		
		self.user_info.insert(&account_id, &totals);
		self.current_sale += token_amount;
		let required_cost = env::storage_usage() - initial_storage_usage;
		let refund = attached_deposit - required_cost as u128;
		
		if refund > 1 {
			Promise::new(self.treasury_id.clone()).transfer(refund);
		}
		
    }
	
	pub fn claim(&mut self) -> bool {
		self.assert_at_least_one_yocto();
		let account_id = env::signer_account_id();
		let staked_tokens = self.user_info.get(&account_id).expect("No climable tokens");
		let current_stimetamp = self.to_sec(env::block_timestamp());
		assert!(
            self.end_time >= current_stimetamp,
            "Not climable date"
        );
		let token_amount = staked_tokens * 10u64.pow(18);
		ext_fungible_token::ft_transfer(
            account_id.clone(),
            U128(token_amount as u128),
            None,
            &self.ft_token_id,
            1,
            GAS_FOR_FT_TRANSFER,
        );
		self.user_info.remove(&account_id);
		true
		
	}
	
	pub fn assert_owner(&self) {
		assert_eq!(
			&env::predecessor_account_id(),
			&self.owner_id,
			"Owner's method"
		);
	}
	
	pub(crate) fn assert_at_least_one_yocto(&self) {
		assert!(
			env::attached_deposit() >= 1,
			"Requires attached deposit of at least 1 yoctoNEAR",
		)
	}

	pub(crate) fn to_sec(&self, timestamp: Timestamp) -> TimestampSec {
		(timestamp / 10u64.pow(9)) as u64
	}

}