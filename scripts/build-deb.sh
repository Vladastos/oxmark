#!/bin/bash

set -e

checks(){
    if [ -z "$OXMARK_VERSION" ]; then
        echo "Please set OXMARK_VERSION."
        exit 1
    fi
}

function main() {
    checks
    create_deb_package
}

function create_deb_package() {
    local deb_directory="./target/deb/oxmark_"$OXMARK_VERSION"-1_amd64"
    mkdir -p "$deb_directory/DEBIAN"
    mkdir -p "$deb_directory/usr/bin"

    cp "target/release/oxmark" "$deb_directory/usr/bin/oxmark"

    create_control_file $deb_directory

    dpkg --build $deb_directory

}

function create_control_file() {
    local deb_directory=$1

    echo "Package: oxmark
Version: $OXMARK_VERSION
Maintainer: Vladislav Parfeniuc
Homepage: https://github.com/Promptorium/oxmark
Architecture: amd64
Description: Oxmark is a simple bookmark manager written in Rust.
" > "$deb_directory/DEBIAN/control"

}

main "$@"