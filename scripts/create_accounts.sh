#!/bin/sh

set -e

near create-account fta.$MASTER_ACCOUNT --masterAccount $MASTER_ACCOUNT
near create-account ftb.$MASTER_ACCOUNT --masterAccount $MASTER_ACCOUNT
near create-account amm.$MASTER_ACCOUNT --masterAccount $MASTER_ACCOUNT
near create-account alice.$MASTER_ACCOUNT --masterAccount $MASTER_ACCOUNT
