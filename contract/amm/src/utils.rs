use near_contract_standards::fungible_token::FungibleToken;
use near_sdk::AccountId;

pub fn add_decimals(value: u128, decimals: u8) -> u128 {
	value * 10_u128.pow(decimals as u32)
}

pub fn remove_decimals(value: u128, decimals: u8) -> u128 {
	value / 10_u128.pow(decimals as u32)
}

// x*y = k
// (x + dx)*(y - dy) = k
// dy = y * dx / (x + dx)
pub fn calc_dy(x: u128, y: u128, dx: u128) -> u128 {
	y - (x * y / (x + dx))
}

pub fn init_token(account_id: &AccountId, prefix: Vec<u8>) -> FungibleToken {
	let mut a = FungibleToken::new(prefix);
	a.internal_register_account(account_id);
	a
}

#[cfg(not(target_arch = "wasm32"))]
#[cfg(test)]
mod tests {
	use super::*;
	#[test]
	fn test_add_decimals() {
		let decimals = add_decimals(50, 3);
		assert_eq!(decimals, 50_000);
	}

	#[test]
	fn test_remove_decimals() {
		let decimals = remove_decimals(50000, 3);
		assert_eq!(decimals, 50);
	}

	#[test]
	fn check_calculator() {
		let x = 1_000_000; // 3 numbers float
		let y = 40_000; // 1 number float
		let max_decimals = 3;
		let y = add_decimals(y, max_decimals - 1);
		let dy = calc_dy(x, y, 1_000_000);
		let dy = remove_decimals(dy, max_decimals - 1);
		assert_eq!(dy, 20_000);
	}
}
