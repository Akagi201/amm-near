use std::cmp::max;

use near_contract_standards::{
	fungible_token::{
		core::FungibleTokenCore, metadata::FungibleTokenMetadata, receiver::FungibleTokenReceiver,
		FungibleToken,
	},
	storage_management::{StorageBalance, StorageBalanceBounds, StorageManagement},
};
use near_sdk::{
	assert_self,
	borsh::{self, BorshDeserialize, BorshSerialize},
	collections::LookupMap,
	env,
	json_types::U128,
	log, near_bindgen,
	serde::{Deserialize, Serialize},
	AccountId, Gas, PanicOnDefault, PromiseOrValue, PromiseResult,
};

mod external;
pub use crate::external::*;

mod utils;
pub use crate::utils::*;

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
	// Contract Owner
	pub owner_id: AccountId,

	// Token A and Token B token_contract:token_interface
	pub tokens: LookupMap<AccountId, FungibleToken>,

	// token_contract:token_metadata
	pub token_metadatas: LookupMap<AccountId, FungibleTokenMetadata>,

	// Liquidity Provider(LP) Token
	pub token_lp: FungibleToken,

	// Token A contract ID
	pub token_a_contract: AccountId,

	// Token B contract ID
	pub token_b_contract: AccountId,

	// Token ratio in the pool (token_a,token_b)
	pub token_ratio: (U128, U128),
}

#[near_bindgen]
#[derive(Serialize, Deserialize, BorshDeserialize, BorshSerialize)]
#[serde(crate = "near_sdk::serde")]
pub struct ContractInfo {
	pub owner_id: AccountId,
	pub token_a_contract: AccountId,
	pub token_b_contract: AccountId,
	pub token_a_meta: FungibleTokenMetadata,
	pub token_b_meta: FungibleTokenMetadata,
	pub token_ratio: (U128, U128),
}

#[near_bindgen]
impl Contract {
	#[init]
	pub fn new(
		owner_id: AccountId,
		token_a_contract: AccountId,
		token_b_contract: AccountId,
	) -> Self {
		assert!(!env::state_exists(), "Pool already initialized");
		let token_a = init_token(&owner_id, b"a".to_vec());
		let token_b = init_token(&owner_id, b"b".to_vec());
		let token_lp = init_token(&owner_id, b"lp".to_vec());
		let mut tokens = LookupMap::new(b"tok".to_vec());
		tokens.insert(&token_a_contract, &token_a);
		tokens.insert(&token_b_contract, &token_b);
		let token_metadatas = LookupMap::new(b"tokdat".to_vec());
		let self_contract_id = env::current_account_id();

		ext_ft::ext(token_a_contract.clone()) // External Contract Token instance
			.ft_metadata() // External Metadata Promise
			.then(
				ext_self::ext(self_contract_id.clone()) // External Contract Self
					.on_ft_metadata(token_a_contract.clone()),
			);

		ext_ft::ext(token_b_contract.clone()) // External Contract Token instance
			.ft_metadata() // External Metadata Promise
			.then(
				ext_self::ext(self_contract_id) // External Contract Self
					.on_ft_metadata(token_b_contract.clone()),
			);

		Self {
			owner_id,
			tokens,
			token_metadatas,
			token_lp,
			token_a_contract,
			token_b_contract,
			token_ratio: (U128(1), U128(1)),
		}
	}

	pub fn swap(
		&mut self,
		buy_token_id: AccountId,
		sell_token_id: AccountId,
		sell_amount: U128,
	) -> U128 {
		if buy_token_id.eq(&sell_token_id) {
			panic!("Tokens can't be equal")
		}

		let mut buy_token = self.tokens.get(&buy_token_id).expect("Buy token not supported");
		let mut sell_token = self.tokens.get(&sell_token_id).expect("Sell token not supported");
		let buy_token_meta =
			self.token_metadatas.get(&buy_token_id).expect("Buy token meta not found");
		let sell_token_meta =
			self.token_metadatas.get(&sell_token_id).expect("Sell token meta not found");
		let pool_owner_id = env::current_account_id();
		let user_account_id = env::predecessor_account_id();

		// Get current state of pool
		let x = sell_token.internal_unwrap_balance_of(&pool_owner_id);
		let y = buy_token.internal_unwrap_balance_of(&pool_owner_id);

		// Send sell_tokens to pool from seller
		sell_token.internal_transfer(&user_account_id, &pool_owner_id, sell_amount.0, None);

		// Convert to the same decimal
		let max_decimals = max(buy_token_meta.decimals, sell_token_meta.decimals);
		let x = add_decimals(x, max_decimals - sell_token_meta.decimals);
		let y = add_decimals(y, max_decimals - buy_token_meta.decimals);

		// Calc buy amount
		let buy_amount = calc_dy(x, y, sell_amount.0);

		// Restore decimal
		let buy_amount = remove_decimals(buy_amount, max_decimals - buy_token_meta.decimals);

		// Send buy value to user buyer
		buy_token.internal_transfer(&pool_owner_id, &user_account_id, buy_amount, None);

		// Update tokens data in lookup map
		self.tokens.insert(&buy_token_id, &buy_token);
		self.tokens.insert(&sell_token_id, &sell_token);

		// update ratio
		let token_a_amount = self
			.tokens
			.get(&self.token_a_contract)
			.unwrap()
			.internal_unwrap_balance_of(&pool_owner_id);
		let token_b_amount = self
			.tokens
			.get(&self.token_b_contract)
			.unwrap()
			.internal_unwrap_balance_of(&pool_owner_id);
		self.token_ratio = (U128(token_a_amount), U128(token_b_amount));

		// Return both amount
		U128::from(buy_amount)
	}

	// Adding tokens to the liquidity pool.
	// Tokens can only be added in proportion to the amount in the pool
	pub fn add_tokens_to_pool(
		&mut self,
		token_a_name: AccountId,
		token_a_amount: U128,
		token_b_name: AccountId,
		token_b_amount: U128,
	) {
		assert!(self.owner_id == env::predecessor_account_id());
		if token_a_name.eq(&token_b_name) {
			panic!("Tokens can't be equal")
		}

		// Get tokens by names
		let mut token_a = self.tokens.get(&token_a_name).expect("Token A not supported");
		let mut token_b = self.tokens.get(&token_b_name).expect("Token B not supported");
		let token_a_meta = self.token_metadatas.get(&token_a_name).expect("Token A meta not found");
		let token_b_meta = self.token_metadatas.get(&token_b_name).expect("Token B meta not found");

		let pool_owner_id = env::current_account_id();
		let payer_id = env::predecessor_account_id();

		// Get current state of pool
		let pool_a_balance = token_a.internal_unwrap_balance_of(&pool_owner_id);
		let pool_b_balance = token_b.internal_unwrap_balance_of(&pool_owner_id);

		// Сonvert to the same decimal
		let max_decimals = max(token_a_meta.decimals, token_b_meta.decimals);

		// We can add tokens to the pool only by proportionally increasing them
		if pool_a_balance * token_b_amount.0 == pool_b_balance * token_a_amount.0 {
			token_a.internal_transfer(&payer_id, &pool_owner_id, token_a_amount.0, None);
			token_b.internal_transfer(&payer_id, &pool_owner_id, token_b_amount.0, None);
			// Calc LP share of added tokens
			let share = add_decimals(token_a_amount.0, max_decimals - token_a_meta.decimals) +
				add_decimals(token_b_amount.0, max_decimals - token_a_meta.decimals);

			// Store LP share
			self.token_lp.internal_deposit(&payer_id, share);
			log!("Share {} has been added to account {}", share, &payer_id);

			// Update tokens data in lookup map
			self.tokens.insert(&token_a_name, &token_a);
			self.tokens.insert(&token_b_name, &token_b);
		} else {
			panic!("incorrect proportions for replenishing the liquidity pool")
		}
		// update ratio
		let token_a_amount = self
			.tokens
			.get(&self.token_a_contract)
			.unwrap()
			.internal_unwrap_balance_of(&pool_owner_id);
		let token_b_amount = self
			.tokens
			.get(&self.token_b_contract)
			.unwrap()
			.internal_unwrap_balance_of(&pool_owner_id);
		self.token_ratio = (U128(token_a_amount), U128(token_b_amount));
	}

	// Here we are excluding all tokens of signed account from
	// liquidity pool and return those tokens back to predecessor_account_id
	// in the right proportion
	pub fn exclude_tokens_from_pool(&mut self, token_a_name: AccountId, token_b_name: AccountId) {
		assert!(self.owner_id == env::predecessor_account_id());
		if token_a_name.eq(&token_b_name) {
			panic!("Tokens can't be equals")
		}
		let mut token_a = self.tokens.get(&token_a_name).expect("Token not supported");
		let mut token_b = self.tokens.get(&token_b_name).expect("Token not supported");

		let pool_owner_id = env::current_account_id();
		let pool_total_a = token_a.internal_unwrap_balance_of(&pool_owner_id);
		let pool_total_b = token_b.internal_unwrap_balance_of(&pool_owner_id);
		let predecessor_account_id = env::predecessor_account_id();

		// Calc all owned user tokens in pool in proportion
		let a = self.token_lp.total_supply * pool_total_a /
			self.token_lp.internal_unwrap_balance_of(&predecessor_account_id);
		let b = self.token_lp.total_supply * pool_total_b /
			self.token_lp.internal_unwrap_balance_of(&predecessor_account_id);

		// Clear user share value
		self.token_lp.internal_withdraw(
			&predecessor_account_id,
			self.token_lp.internal_unwrap_balance_of(&predecessor_account_id),
		);
		// Transfer tokens from pool to user wallet
		token_a.internal_transfer(&env::current_account_id(), &predecessor_account_id, a, None);
		token_b.internal_transfer(&env::current_account_id(), &predecessor_account_id, b, None);
		// Update tokens data in lookup map
		self.tokens.insert(&token_a_name, &token_a);
		self.tokens.insert(&token_b_name, &token_b);
		// update ratio
		let token_a_amount = self
			.tokens
			.get(&self.token_a_contract)
			.unwrap()
			.internal_unwrap_balance_of(&pool_owner_id);
		let token_b_amount = self
			.tokens
			.get(&self.token_b_contract)
			.unwrap()
			.internal_unwrap_balance_of(&pool_owner_id);
		self.token_ratio = (U128(token_a_amount), U128(token_b_amount));
	}

	#[payable]
	pub fn withdraw_tokens(&mut self, token_name: AccountId, amount: U128) {
		let account_id = env::predecessor_account_id();
		if !self.tokens.contains_key(&token_name) {
			panic!("Token not supported");
		}
		ext_ft::ext(token_name.clone())
			.with_static_gas(Gas(5 * TGAS))
			.ft_transfer(account_id, amount, None)
			.then(
				ext_self::ext(env::current_account_id())
					.with_static_gas(Gas(5 * TGAS))
					.withdraw_tokens_callback(token_name.to_string(), amount),
			);
	}

	#[private]
	pub fn withdraw_tokens_callback(&mut self, token_name: AccountId, amount: U128) {
		match env::promise_result(0) {
			PromiseResult::NotReady => unreachable!(),
			PromiseResult::Failed => "error!".to_string(),
			PromiseResult::Successful(_) => {
				// Get the user who sent the tokens
				let account_id = env::signer_account_id();
				let mut token = self.tokens.get(&token_name).unwrap();

				// Clear sent tokens value
				token.internal_withdraw(&account_id, amount.0);

				// Update token in lookup map
				self.tokens.insert(&token_name, &token);
				"Ok".to_string()
			},
		};
	}

	#[private]
	pub fn on_ft_metadata(
		&mut self,
		contract_id: AccountId,
		#[callback] metadata: FungibleTokenMetadata,
	) {
		assert_self();
		log!("on_ft_metadata: contract_id: {}", contract_id);

		if !self.tokens.contains_key(&contract_id) {
			panic!("Token not supported");
		}

		self.token_metadatas.insert(&contract_id, &metadata);
	}

	pub fn ft_balance_of(&self, token_name: AccountId, account_id: AccountId) -> U128 {
		if token_name == env::current_account_id() {
			self.token_lp.ft_balance_of(account_id)
		} else {
			self.tokens
				.get(&token_name)
				.expect("Token not supported")
				.ft_balance_of(account_id)
		}
	}

	pub fn contract_info(&self) -> ContractInfo {
		ContractInfo {
			owner_id: self.owner_id.clone(),
			token_a_contract: self.token_a_contract.clone(),
			token_b_contract: self.token_b_contract.clone(),
			token_a_meta: self.token_metadatas.get(&self.token_a_contract).unwrap(),
			token_b_meta: self.token_metadatas.get(&self.token_b_contract).unwrap(),
			token_ratio: self.token_ratio,
		}
	}

	#[payable]
	#[allow(dead_code)]
	pub fn storage_deposit(
		&mut self,
		token_name: AccountId,
		account_id: AccountId,
		registration_only: Option<bool>,
	) {
		if token_name == env::current_account_id() {
			self.token_lp.storage_deposit(Some(account_id), registration_only);
		} else {
			let mut token = self.tokens.get(&token_name).unwrap();
			token.storage_deposit(Some(account_id), registration_only);
			// self.tokens.insert(&token_name, &token);
		}
	}

	#[payable]
	#[allow(dead_code)]
	#[allow(clippy::let_and_return)]
	fn storage_withdraw(&mut self, token_name: AccountId, amount: Option<U128>) -> StorageBalance {
		if token_name == env::current_account_id() {
			self.token_lp.storage_withdraw(amount)
		} else {
			let mut token = self.tokens.get(&token_name).unwrap();
			let storage_balance = token.storage_withdraw(amount);
			// self.tokens.insert(&token_name, &token);
			storage_balance
		}
	}

	#[payable]
	#[allow(dead_code)]
	fn storage_unregister(&mut self, token_name: AccountId, force: Option<bool>) -> bool {
		if token_name == env::current_account_id() {
			if let Some((_, _)) = self.token_lp.internal_storage_unregister(force) {
				return true
			}
		} else {
			let mut token = self.tokens.get(&token_name).unwrap();
			if let Some((_, _)) = token.internal_storage_unregister(force) {
				// self.tokens.insert(&token_name, &token);
				return true
			}
		}
		false
	}

	#[allow(dead_code)]
	fn storage_balance_bounds(&self, token_name: AccountId) -> StorageBalanceBounds {
		if token_name == env::current_account_id() {
			self.token_lp.storage_balance_bounds()
		} else {
			let token = self.tokens.get(&token_name).unwrap();
			token.storage_balance_bounds()
		}
	}

	#[allow(dead_code)]
	fn storage_balance_of(
		&self,
		token_name: AccountId,
		account_id: AccountId,
	) -> Option<StorageBalance> {
		if token_name == env::current_account_id() {
			self.token_lp.storage_balance_of(account_id)
		} else {
			let token = self.tokens.get(&token_name).unwrap();
			token.storage_balance_of(account_id)
		}
	}
}

// Use FT.ft_transfer_call to send tokens from FT to the AMM Pool
#[near_bindgen]
impl FungibleTokenReceiver for Contract {
	fn ft_on_transfer(
		&mut self,
		sender_id: AccountId,
		amount: U128,
		#[allow(unused_variables)] msg: String,
	) -> PromiseOrValue<U128> {
		let token_name = &env::predecessor_account_id();
		let mut token = self.tokens.get(token_name).expect("Token not supported");
		token.internal_deposit(&sender_id, amount.0);
		self.tokens.insert(token_name, &token);
		PromiseOrValue::Value(U128::from(0_u128))
	}
}
