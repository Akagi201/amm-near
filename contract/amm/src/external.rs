use near_contract_standards::fungible_token::metadata::FungibleTokenMetadata;
use near_sdk::{ext_contract, json_types::U128, AccountId, Balance};

pub const TGAS: u64 = 1_000_000_000_000;

// Validator interface, for cross-contract calls

// Define an interface for callbacks
#[ext_contract(ext_self)]
pub trait SelfContract {
	fn on_get_metadata(
		&mut self,
		contract_id: AccountId,
		#[callback] metadata: FungibleTokenMetadata,
	);
	fn withdraw_tokens_callback(&mut self, token_name: String, amount: U128);
}

// FT Contract interface
#[ext_contract(ext_ft)]
pub trait FtContract {
	fn create_wallet(&mut self, sender_id: AccountId, amount: Balance);
	fn get_metadata(&self) -> FungibleTokenMetadata;
	fn ft_transfer(&mut self, receiver_id: AccountId, amount: U128, memo: Option<String>);
	fn transfer_from(&mut self, sender_id: AccountId, receiver_id: AccountId, amount: Balance);
}
