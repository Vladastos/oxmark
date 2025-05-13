#!/bin/bash

function main(){
    
    # Check if oxmark is already installed
    if  command -v oxmark >/dev/null; then
        echo "Oxmark is already installed"
        exit 0
    fi

    if is_using_apt; then
         apt_install
    else
         generic_install
    fi
}
function apt_install() {
    
    echo "Installing Promptorium using apt..."

    # Add promptorium gpg key if it doesn't exist
    if [[ ! -f /etc/apt/keyrings/promptorium-gpg.public ]]; then
        echo "Adding promptorium gpg key..."
        sudo mkdir -p /etc/apt/keyrings
        sudo chmod 755 /etc/apt/keyrings
        local gpg_key
        gpg_key=$(curl -s https://apt.promptorium.org/gpg-key.public)
        echo "$gpg_key" | sudo tee /etc/apt/keyrings/promptorium-gpg.public > /dev/null
    else
        echo "promptorium gpg key already exists"
    
    fi

    # Add promptorium apt repository if it doesn't exist
    if [[ ! -f /etc/apt/sources.list.d/promptorium.list ]]; then
        echo "Adding promptorium apt repository..."
        local repository_url
        repository_url="deb [arch=amd64 signed-by=/etc/apt/keyrings/promptorium-gpg.public] https://apt.promptorium.org/ unstable main"

        echo "$repository_url" | sudo tee /etc/apt/sources.list.d/promptorium.list > /dev/null
    else
        echo "promptorium apt repository already exists"
    fi

    # Install oxmark
    echo "Updating apt repositories..."
    sudo apt update
    sudo apt install oxmark -y

}

function generic_install() {

    echo "Installing Oxmark..."
    
    local url
    url=$(curl https://api.github.com/repos/Promptorium/oxmark/releases/latest \
    | grep "browser_download_url.*linux_amd64" | cut -d : -f 2,3 | tr -d \" )
    
    if [[ -z $url ]]; then
        echo "Failed to get download URL"
        exit 1
    fi

    echo "Downloading oxmark binary..."
    sudo wget -O /usr/local/bin/oxmark $url
    sudo chmod +x /usr/local/bin/oxmark

    echo "Oxmark installed successfully"

}

function is_using_apt() {
    if [[ -n $(command -v apt-get 2>/dev/null) ]]; then
        return 0
    fi
    if [[ -n $(command -v apt 2>/dev/null) ]]; then
        return 0
    fi
    return 1
}

main

