#!/bin/bash

#####################################

## Shell Script to Build Rust Lambdas   

#####################################
echo "Start Building Task"
echo " "

echo "Enter source directory"
cd ./src
echo " "

#######################
#### BUILD LAMBDAS ####
#######################

echo "Building 'core' lambda"
echo " "
cd ./cf-user_core
cargo build --release
cd ..
echo "'core' lambda build complete"

## Duplicate the following lines when appending a new lambda function to the cf-user directory

#### CREATE USER ####
echo "building 'create-user' lambda"
echo " "
cd ./cf-user_create-user
rustup target add aarch64-unknown-linux-gnu
cargo lambda build --release --output-format zip --arm64
cd ..
echo "'create-user' lambda build complete"
#####################

#### GET USER ####
echo "building 'get-user' lambda"
echo " "
cd ./cf-user_get-user
rustup target add aarch64-unknown-linux-gnu
cargo lambda build --release --output-format zip --arm64
cd ..
echo "'get-user' lambda build complete"
#####################

#### UPDATE USER ####
echo "building 'update-user' lambda"
echo " "
cd ./cf-user_update-user
rustup target add aarch64-unknown-linux-gnu
cargo lambda build --release --output-format zip --arm64
cd ..
echo "'update-user' lambda build complete"
#####################

#### DELETE USER ####
echo "building 'delete-user' lambda"
echo " "
cd ./cf-user_delete-user
rustup target add aarch64-unknown-linux-gnu
cargo lambda build --release --output-format zip --arm64
cd ..
echo "'delete-user' lambda build complete"
#####################

#### LIST USERS ####
echo "building 'list-users' lambda"
echo " "
cd ./cf-user_list-users
rustup target add aarch64-unknown-linux-gnu
cargo lambda build --release --output-format zip --arm64
cd ..
echo "'list-users' lambda build complete"
#####################

printf "\nBuilding Task Complete!"