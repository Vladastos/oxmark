#!/bin/bash

function checks(){
	if [ -z "$RUSTMARKS_VERSION" ]; then
		echo "Please set RUSTMARKS_VERSION."
		exit 1
	fi
}

function update_cargo_file() {

	# Update Cargo.toml
	sed -i "s|version = \"0.0.0\"|version = \"$RUSTMARKS_VERSION\"|g" Cargo.toml
}

function main() {

	checks
	echo "Building rustmarks version $RUSTMARKS_VERSION"

	update_cargo_file
	cargo build --release

}

main
