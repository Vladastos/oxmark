use std::path::PathBuf;
use clap::{Parser, Subcommand};
mod sqlite_repository;

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Some(Commands::Add {
            path,
            name,
            description,
        }) => {
            add_cmd(path, name, description);
        }
        Some(Commands::Delete { path}) => {
            delete_cmd(path);
        }
        Some(Commands::Update {
            id,
            path,
            name,
            description,
        }) => {
            update_cmd(id, path, name, description);
        }
        Some(Commands::List { pathsonly }) => {
            list(pathsonly);
        }
        Some(Commands::Command {}) => {
            command();
        }
        None => {
            default();
        }
    }
}

/*
 * Commands
 */

fn add_cmd(path: String, name: Option<String>, description: Option<String>) {
    println!("Adding bookmark: {} {} {}", path, name.clone().unwrap_or("".to_string()), description.clone().unwrap_or("".to_string()));
    let repo = sqlite_repository::SqliteRepository::new();
    let bookmark = sqlite_repository::Bookmark {
        id: 0,
        name: name.clone(),
        path: Some(path),
        description: description,
    };
    let result = repo.add_bookmark(bookmark);
    if let Err(e) = result {
        println!("Error: {}", e);
        return;
    }
}

fn delete_cmd(path: String) {
    println!("Removing bookmark: {}", path);
}

fn update_cmd(id: i32, path: Option<String>, name: Option<String>, description: Option<String>) {
    println!("Updating bookmark: {} {} {} {}", id, path.unwrap_or("".to_string()), name.unwrap_or("".to_string()), description.unwrap_or("".to_string()));
}

fn list(pathsonly: bool) {

    let repo = sqlite_repository::SqliteRepository::new();
    let result = repo.list_bookmarks();

    if let Err(e) = result {
        println!("Error: {}", e);
        return;
    }

    let bookmarks = result.unwrap();

    println!("Number of bookmarks: {}", bookmarks.len());
    
    // TODO: Print the bookmarks
}

fn command() {
    println!("Printing command");
}

fn default() {
    list(false);
}

/*
 * Structs
 */

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    /// Optional name to operate on
    name: Option<String>,

    /// Sets a custom config file
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,

    /// Turn debugging information on
    #[arg(short, long, action = clap::ArgAction::Count)]
    debug: u8,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    // Add a bookmark
    Add {
        /// The path of the bookmark
        path: String,

        /// The name of the bookmark
        #[arg(short, long)]
        name: Option<String>,

        /// The description of the bookmark
        #[arg(short, long)]
        description: Option<String>,
    },

    // Delete a bookmark
    Delete {
        /// The id of the bookmark
        path: String,
    },

    Update {
        /// The id of the bookmark
        id: i32,

        /// The path of the bookmark
        #[arg(short, long)]
        path: Option<String>,

        /// The name of the bookmark
        #[arg(short, long)]
        name: Option<String>,

        /// The description of the bookmark
        #[arg(short, long)]
        description: Option<String>,
    },

    // List all bookmarks
    List {
        #[arg(short, long, action = clap::ArgAction::SetTrue)]
        pathsonly: bool,
    },

    // Print the command for selected bookmark
    Command {},
}