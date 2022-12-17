# amm-near

[![Rust CI](https://github.com/Akagi201/amm-near/actions/workflows/rust.yml/badge.svg)](https://github.com/Akagi201/amm-near/actions/workflows/rust.yml) [![Super Linter](https://github.com/Akagi201/amm-near/actions/workflows/super_linter.yml/badge.svg)](https://github.com/Akagi201/amm-near/actions/workflows/super_linter.yml)

A simple AMM swap contract in NEAR.

## Usage

```sh
make help # get command list
```

## Build all contracts

```sh
make build
```

## Deploy

```sh
direnv allow # enable .env vars, modify .envrc to your requirements
make accounts
make deploy_fta
make deploy_ftb
make deploy_amm
```

## Tests

Contract unit test

```sh
make test
```

## Docs

* [Fungible Token Standards](https://nomicon.io/Standards/Tokens/FungibleToken/Core)
* [Fungible Tokens Zero to Hero](https://docs.near.org/tutorials/fts/introduction)
