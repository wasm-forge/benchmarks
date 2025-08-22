#!/bin/bash

set -e

# add some persons to the database
export  USER_COUNT=10000:nat64
export ORDER_COUNT=200000:nat64

#echo create tables
#dfx canister call sql-users-orders-backend create_tables 


#echo adding users
#dfx canister call sql-users-orders-backend add_users "(0:nat64, $USER_COUNT)"

#echo adding orders
#dfx canister call sql-users-orders-backend add_orders "(0:nat64, $ORDER_COUNT, $USER_COUNT)"

# this causes an error when tmp pages are stored in file instead of memory
echo create indices
dfx canister call sql-users-orders-backend create_indices 

