#!/bin/bash

set -e

function checks(){
	if [ -z "$OXMARK_VERSION" ]; then
		echo "Please set OXMARK_VERSION."
		exit 1
	fi
}

function update_cargo_file() {

	# Update Cargo.toml

	cp Cargo.toml Cargo.toml.tmp
	sed -i "s/OXMARK_VERSION/$OXMARK_VERSION/g" Cargo.toml
}

function restore_cargo_file() {

	# Restore Cargo.toml

	cp Cargo.toml.tmp Cargo.toml
	rm Cargo.toml.tmp
}

function main() {

	checks
	echo "Building oxmark version $OXMARK_VERSION"

	update_cargo_file
	cargo build --release

	restore_cargo_file
}

main
