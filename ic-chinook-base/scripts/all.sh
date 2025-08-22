#!/bin/bash

#dfx stop
#dfx start --clean --background

dfx deploy

rm chinook.db
rm chinook.zip

# launch this script from the root project folder with ./scripts/all.sh

echo "download sample database"
./scripts/sample_download.sh

echo "upload database to the server"
./scripts/upload_db.sh

echo "set database cache size"
./scripts/set_page_size.sh

echo "fill database with 1000000 records"
./scripts/fill_data.sh

echo "run benchmarks"
./scripts/bench.sh


