#!/bin/sh

set -e

# Setup FTA and FTB contracts

near call fta.$MASTER_ACCOUNT new '{
  "owner_id": "fta.'$MASTER_ACCOUNT'",
  "total_supply": "10000000",
  "metadata": {
    "spec": "ft-1.0.0",
    "name": "Fungible Token A",
    "symbol": "FTA",
    "icon": null,
    "reference": null,
    "reference_hash": null,
    "decimals": 4
  }}' --accountId fta.$MASTER_ACCOUNT
# view on https://testnet.nearblocks.io/zh-cn/token/fta.decode.testnet
near view fta.$MASTER_ACCOUNT ft_metadata

near call fta.$MASTER_ACCOUNT storage_deposit '{
  "account_id": "alice.'$MASTER_ACCOUNT'"
}' --accountId fta.$MASTER_ACCOUNT --deposit 1 --gas 25000000000000

near call fta.$MASTER_ACCOUNT storage_deposit '{
  "account_id": "amm.'$MASTER_ACCOUNT'"
}' --accountId fta.$MASTER_ACCOUNT --deposit 1 --gas 25000000000000

near call fta.$MASTER_ACCOUNT ft_transfer '{
  "receiver_id": "alice.'$MASTER_ACCOUNT'",
  "amount": "1000000"
}' --accountId fta.$MASTER_ACCOUNT --depositYocto 1

near call ftb.$MASTER_ACCOUNT new '{
  "owner_id": "ftb.'$MASTER_ACCOUNT'",
  "total_supply": "10000000",
  "metadata": {
    "spec": "ft-1.0.0",
    "name": "Fungible Token B",
    "symbol": "FTB",
    "icon": null,
    "reference": null,
    "reference_hash": null,
    "decimals": 4
  }}' --accountId ftb.$MASTER_ACCOUNT
# view https://testnet.nearblocks.io/zh-cn/token/ftb.decode.testnet
near view ftb.$MASTER_ACCOUNT ft_metadata

near call ftb.$MASTER_ACCOUNT storage_deposit '{
  "account_id": "alice.'$MASTER_ACCOUNT'"
}' --accountId ftb.$MASTER_ACCOUNT --deposit 1 --gas 25000000000000

near call ftb.$MASTER_ACCOUNT storage_deposit '{
  "account_id": "amm.'$MASTER_ACCOUNT'"
}' --accountId ftb.$MASTER_ACCOUNT --deposit 1 --gas 25000000000000

# Setup AMM contract

near call amm.$MASTER_ACCOUNT new '{
  "owner_id": "alice.'$MASTER_ACCOUNT'",
  "token_a_contract": "fta.'$MASTER_ACCOUNT'",
  "token_b_contract": "ftb.'$MASTER_ACCOUNT'"
}' --accountId amm.$MASTER_ACCOUNT

# view init state
near view amm.$MASTER_ACCOUNT contract_info

near call amm.$MASTER_ACCOUNT storage_deposit '{"token_name":"fta.'$MASTER_ACCOUNT'","account_id": "alice.'$MASTER_ACCOUNT'"}' --accountId amm.$MASTER_ACCOUNT --deposit 1 --gas 25000000000000;
near call amm.$MASTER_ACCOUNT storage_deposit '{"token_name":"fta.'$MASTER_ACCOUNT'","account_id": "amm.'$MASTER_ACCOUNT'"}' --accountId amm.$MASTER_ACCOUNT --deposit 1 --gas 25000000000000;
near call amm.$MASTER_ACCOUNT storage_deposit '{"token_name":"ftb.'$MASTER_ACCOUNT'","account_id": "alice.'$MASTER_ACCOUNT'"}' --accountId amm.$MASTER_ACCOUNT --deposit 1 --gas 25000000000000;
near call amm.$MASTER_ACCOUNT storage_deposit '{"token_name":"ftb.'$MASTER_ACCOUNT'","account_id": "amm.'$MASTER_ACCOUNT'"}' --accountId amm.$MASTER_ACCOUNT --deposit 1 --gas 25000000000000;

near call amm.$MASTER_ACCOUNT storage_deposit '{"token_name":"amm.'$MASTER_ACCOUNT'","account_id": "alice.'$MASTER_ACCOUNT'"}' --accountId amm.$MASTER_ACCOUNT --deposit 1 --gas 25000000000000;

near call fta.$MASTER_ACCOUNT ft_transfer '{
  "receiver_id": "alice.'$MASTER_ACCOUNT'",
  "amount": "1000000"
}' --accountId fta.$MASTER_ACCOUNT --depositYocto 1

near call ftb.$MASTER_ACCOUNT ft_transfer '{
  "receiver_id": "alice.'$MASTER_ACCOUNT'",
  "amount": "1000000"
}' --accountId ftb.$MASTER_ACCOUNT --depositYocto 1

# near call fta.$MASTER_ACCOUNT ft_transfer '{
#   "receiver_id": "amm.'$MASTER_ACCOUNT'",
#   "amount": "1000000"
# }' --accountId fta.$MASTER_ACCOUNT --depositYocto 1

# near call ftb.$MASTER_ACCOUNT ft_transfer '{
#   "receiver_id": "amm.'$MASTER_ACCOUNT'",
#   "amount": "1000000"
# }' --accountId ftb.$MASTER_ACCOUNT --depositYocto 1

near call fta.$MASTER_ACCOUNT ft_transfer_call '{
  "receiver_id": "amm.'$MASTER_ACCOUNT'",
  "amount": "500000",
  "msg": ""
  }' --accountId alice.$MASTER_ACCOUNT --depositYocto 1 --gas 50000000000000

near call ftb.$MASTER_ACCOUNT ft_transfer_call '{
  "receiver_id": "amm.'$MASTER_ACCOUNT'",
  "amount": "500000",
  "msg": ""
  }' --accountId alice.$MASTER_ACCOUNT --depositYocto 1 --gas 50000000000000

near call amm.$MASTER_ACCOUNT add_tokens_to_pool '{"token_a_name": "fta.'$MASTER_ACCOUNT'", "token_a_amount": "400000", "token_b_name": "ftb.'$MASTER_ACCOUNT'", "token_b_amount": "200000"}' --accountId alice.$MASTER_ACCOUNT --gas 25000000000000

near view amm.$MASTER_ACCOUNT contract_info
near view amm.$MASTER_ACCOUNT ft_balance_of '{"token_name": "fta.'$MASTER_ACCOUNT'", "account_id": "alice.'$MASTER_ACCOUNT'"}'
near view amm.$MASTER_ACCOUNT ft_balance_of '{"token_name": "ftb.'$MASTER_ACCOUNT'", "account_id": "alice.'$MASTER_ACCOUNT'"}'

near call amm.$MASTER_ACCOUNT swap '{
    "buy_token_id": "fta.'$MASTER_ACCOUNT'",
    "sell_token_id": "ftb.'$MASTER_ACCOUNT'",
    "sell_amount": "100000"
  }' --accountId alice.$MASTER_ACCOUNT --gas 25000000000000

near view amm.$MASTER_ACCOUNT contract_info

near view amm.$MASTER_ACCOUNT ft_balance_of '{"token_name": "fta.'$MASTER_ACCOUNT'", "account_id": "alice.'$MASTER_ACCOUNT'"}'
near view amm.$MASTER_ACCOUNT ft_balance_of '{"token_name": "ftb.'$MASTER_ACCOUNT'", "account_id": "alice.'$MASTER_ACCOUNT'"}'
