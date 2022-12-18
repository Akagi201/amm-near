#!/bin/sh

set -e

near deploy fta.$MASTER_ACCOUNT --wasmFile res/ft.wasm
near deploy ftb.$MASTER_ACCOUNT --wasmFile res/ft.wasm
near deploy amm.$MASTER_ACCOUNT --wasmFile res/amm.wasm
