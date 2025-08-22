#!/bin/bash

set -e

dfx canister deposit-cycles --all 1000000000000

# add customers 
echo "creating index"

dfx canister call chinook_base create_chinook_indices

echo "adding customers"


TOTAL=100000000000
COUNTER=1000000

PER=100000

while [ $COUNTER -lt $TOTAL ];
do

    dfx canister deposit-cycles --all 200000000000

    CUR=`expr $COUNTER + $PER`
    echo "--- ${CUR} ---"

    dfx canister call chinook_base add_customers "($COUNTER)"

    db_size=`dfx canister call chinook_base get_db_size`
    echo "db_size: $db_size"

    count=`dfx canister call chinook_base query '("select count(*) from customers")'`
    echo "count: $count" 

    # extract the number (between quotes)
    num=$(echo "$count" | grep -oP '"\K[0-9]+(?=")')

    # check if it's greater than 1000000
    if (( num > 1000000 )); then
        exit 0
    fi    

    COUNTER=`expr $COUNTER + $PER`

done

