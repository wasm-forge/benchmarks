#!/bin/bash

set -e


db_size=`dfx canister call chinook_base get_db_size`
echo "db_size: $db_size"

dfx canister call chinook_base query '("pragma page_size=4096")'

dfx canister call chinook_base query '("pragma cache_size=10000")'

dfx canister call chinook_base query '("vacuum")'

db_size=`dfx canister call chinook_base get_db_size`
echo "db_size: $db_size"

