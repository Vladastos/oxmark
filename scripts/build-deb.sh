#!/bin/bash

set -e

checks(){
    if [ -z "$RUSTMARKS_VERSION" ]; then
        echo "Please set RUSTMARKS_VERSION."
        exit 1
    fi
}

function main() {
    checks
    create_deb_package
}

function create_deb_package() {
    local deb_directory="./target/deb/rustmarks_"$RUSTMARKS_VERSION"-1_amd64"
    mkdir -p "$deb_directory/DEBIAN"
    mkdir -p "$deb_directory/usr/bin"

    cp "target/release/rustmarks" "$deb_directory/usr/bin/rustmarks"

    create_control_file $deb_directory

    dpkg --build $deb_directory

}

function create_control_file() {
    local deb_directory=$1

    echo "Package: rustmarks
Version: $RUSTMARKS_VERSION
Maintainer: Vladislav Parfeniuc
Homepage: https://github.com/Promptorium/rustmarks
Architecture: amd64
Description: Rustmarks is a simple bookmark manager written in Rust.
" > "$deb_directory/DEBIAN/control"

}

main "$@"