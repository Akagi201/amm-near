use near_contract_standards::fungible_token::metadata::FungibleTokenMetadata;
use near_sdk::{ext_contract, json_types::U128, AccountId};

pub const TGAS: u64 = 1_000_000_000_000;

// Validator interface, for cross-contract calls

// Define an interface for callbacks
#[ext_contract(ext_self)]
pub trait SelfContract {
	fn on_ft_metadata(
		&mut self,
		contract_id: AccountId,
		#[callback] metadata: FungibleTokenMetadata,
	);
	fn withdraw_tokens_callback(&mut self, token_name: String, amount: U128);
}

// FT Contract interface
#[ext_contract(ext_ft)]
pub trait FtContract {
	fn ft_metadata(&self) -> FungibleTokenMetadata;
	fn ft_transfer(&mut self, receiver_id: AccountId, amount: U128, memo: Option<String>);
}
