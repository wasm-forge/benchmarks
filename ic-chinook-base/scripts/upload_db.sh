#!/bin/bash

set -e

file=chinook.db

if [[ ! -f "$file" ]]; then
    echo "Error: File '$file' does not exist, download it via './scripts/sample_download.sh'." >&2
    exit 1
fi

# prepare arguments
echo -n '(blob "' > args.txt
xxd -p -c 10000000 $file | tr -d '\n' | sed 's/\(..\)/\\\1/g' >>  args.txt
echo -n '" )' >> args.txt

# upload DB
dfx canister call chinook_base upload_database --argument-file args.txt



