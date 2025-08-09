#!/bin/sh

set -e

dfx canister call chinook_base download_database | sed -En 's/.*"([^"]*)".*/\1/p' | xxd -p -r > canister_base.db

