#!/bin/sh

set -e

near delete --force true fta.$MASTER_ACCOUNT $MASTER_ACCOUNT --verbose true
near delete --force true ftb.$MASTER_ACCOUNT $MASTER_ACCOUNT --verbose true
near delete --force true amm.$MASTER_ACCOUNT $MASTER_ACCOUNT --verbose true
near delete --force true alice.$MASTER_ACCOUNT $MASTER_ACCOUNT --verbose true
