#!/bin/bash

set -e

# reset DB cache
dfx canister call chinook_base close_database
# 1st run
dfx canister call chinook_base query '("SELECT COUNT(*) FROM customers")'
# 2nd run
dfx canister call chinook_base query '("SELECT COUNT(*) FROM customers")'


# reset DB cache
dfx canister call chinook_base close_database
# 1st run
dfx canister call chinook_base query '("SELECT firstname, lastname, email FROM customers WHERE customerid=900000")'
# 2nd run
dfx canister call chinook_base query '("SELECT firstname, lastname, email FROM customers WHERE customerid=900000")'


# reset DB cache
dfx canister call chinook_base close_database
# 1st run
dfx canister call chinook_base query '("SELECT firstname, lastname, email FROM customers WHERE firstname = \"2912169customer_name2912169\"")'
# 2nd run
dfx canister call chinook_base query '("SELECT firstname, lastname, email FROM customers WHERE firstname = \"2912169customer_name2912169\"")'


# reset DB cache
dfx canister call chinook_base close_database
# 1st run
dfx canister call chinook_base query '("SELECT firstname, lastname, email FROM customers WHERE firstname = \"1\"")'
# 2nd run
dfx canister call chinook_base query '("SELECT firstname, lastname, email FROM customers WHERE firstname = \"1\"")'


# reset DB cache
dfx canister call chinook_base close_database
# 1st run
dfx canister call chinook_base query '("SELECT firstname, lastname, email FROM customers WHERE customerid>900000 and customerid<900050")'
# 2nd run
dfx canister call chinook_base query '("SELECT firstname, lastname, email FROM customers WHERE customerid>900000 and customerid<900050")'

# reset DB cache
dfx canister call chinook_base close_database
# 1st run
dfx canister call chinook_base query '("SELECT count(*) FROM customers WHERE firstname>=\"1\" and firstname<\"2\"")'
# 2nd run
dfx canister call chinook_base query '("SELECT count(*) FROM customers WHERE firstname>=\"1\" and firstname<\"2\"")'


