# Rustmarks

Rustmarks is a command-line tool for managing bookmarks and navigating through them.

## Installation

To install Rustmarks, run the following command:
```bash
bash <(curl -s https://raw.githubusercontent.com/Promptorium/rustmarks/main/install.sh)
```
It will download it using apt if you are using a debian based system, otherwise it will download the binary directly and place it in `/usr/local/bin`.

## Usage

After installing Rustmarks, run `rustmarks init` to initialize the database and add the shell script to your `~/.bashrc` or `~/.zshrc` file.

Then you can use the `bk` command to navigate through your bookmarks.

To add a bookmark, use the `add` command:

```bash
rustmarks add /path/to/bookmark <name> <description>
```