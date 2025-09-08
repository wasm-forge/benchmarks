#!/bin/bash


dfx canister stop sql-users-orders-backend

dfx canister snapshot create sql-users-orders-backend

dfx canister snapshot download --dir ./snaps sql-users-orders-backend 0000000000000000ffffffffff9000010101



