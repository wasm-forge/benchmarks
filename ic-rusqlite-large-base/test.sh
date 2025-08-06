#!/bin/bash

set -e

# add some persons to the database
export  USER_COUNT=100000:nat64
export ORDER_COUNT=100:nat64

echo adding users
dfx canister call sql-users-orders-backend add_users "(0:nat64, $USER_COUNT)"

echo adding orders
dfx canister call sql-users-orders-backend add_orders "(0:nat64, $ORDER_COUNT, $USER_COUNT)"

