use std::io::{Read, Write};

use clap::{Parser, Subcommand};
mod sqlite;
mod ui;

fn main() {
    let cli = Cli::parse();

    let sqlite_service_result = sqlite::SqliteService::new();

    if let Err(e) = sqlite_service_result {
        println!("{}", e.message());
        std::process::exit(1);
    }
    let sqlite_service = sqlite_service_result.unwrap();

    let mut app = ui::App::new();

    match cli.command {
        Some(Commands::Add {
            path,
            name,
            description,
        }) => {
            sqlite_service
                .create(path, name, description)
                .unwrap_or_else(|e| {
                    println!("{}", e.message());
                    std::process::exit(1);
                });
        }
        Some(Commands::Delete { path }) => {
            sqlite_service.delete(path).unwrap_or_else(|e| {
                println!("{}", e.message());
                std::process::exit(1);
            });
        }
        Some(Commands::Update {
            id,
            path,
            name,
            description,
        }) => {
            sqlite_service
                .update(id, path, name, description)
                .unwrap_or_else(|e| {
                    println!("{}", e.message());
                    std::process::exit(1);
                });
        }
        Some(Commands::List { pathsonly }) => {
            sqlite_service.get_all(pathsonly).unwrap_or_else(|e| {
                println!("{}", e.message());
                std::process::exit(1);
            });
        }
        Some(Commands::Command {}) => {
            app.run(true).unwrap_or_else(|e| {
                println!("{}", e);
                std::process::exit(1);
            });
        }
        Some(Commands::Init {}) => {
            let function_string = "\n# Rustmarks \nfunction bk() { if [ -z \"$1\" ]; then \"$(rustmarks command)\"; else rustmarks \"$@\"; fi }";

            // Look for .bashrc or .zshrc
            let home = std::env::var("HOME").unwrap_or("/home".to_string());
            let bashrc = format!("{}/.bashrc", home);
            let zshrc = format!("{}/.zshrc", home);
            let bashrc_exists = std::path::Path::new(&bashrc).exists();
            let zshrc_exists = std::path::Path::new(&zshrc).exists();

            // TODO: Check if line already exists first

            if bashrc_exists {
                let mut file = std::fs::OpenOptions::new()
                    .append(true)
                    .read(true)
                    .open(&bashrc)
                    .unwrap();

                let mut buf = String::new();
                file.read_to_string(&mut buf).unwrap();

                if !buf.contains(function_string) {
                    println!("Adding line to .bashrc");
                    file.write_all(function_string.as_bytes()).unwrap();
                } else {
                    println!("Line already exists in .bashrc");
                }
            }
            if zshrc_exists {
                let mut file = std::fs::OpenOptions::new()
                    .append(true)
                    .read(true)
                    .open(&zshrc)
                    .unwrap();

                let mut buf = String::new();
                file.read_to_string(&mut buf).unwrap();

                if !buf.contains(function_string) {
                    println!("Adding line to .zshrc");
                    file.write_all(function_string.as_bytes()).unwrap();
                } else {
                    println!("Line already exists in .zshrc");
                }
            }
            println!(
                "Done. After restarting your terminal use bk command to start using rustmarks"
            );
        }
        None => {
            app.run(false).unwrap_or_else(|e| {
                println!("{}", e);
                std::process::exit(1);
            });
        }
    }
}

/*
 * Structs
 */

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    /// Optional name to operate on
    name: Option<String>,

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
        name: Option<String>,

        /// The description of the bookmark
        description: Option<String>,
    },

    // Delete a bookmark
    Delete {
        /// The path of the bookmark
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

    // Initialize rustmarks
    Init {},
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Bookmark {
    pub id: Option<i32>,
    pub name: Option<String>,
    pub path: Option<String>,
    pub description: Option<String>,
}
impl Bookmark {
    pub fn new(
        name: Option<String>,
        path: Option<String>,
        description: Option<String>,
    ) -> Bookmark {
        // Canonicalize the path, meaning that it will be an absolute path
        let abs_path = if let Some(path) = path {
            std::path::PathBuf::from(path)
                .canonicalize()
                .unwrap()
                .to_str()
                .unwrap()
                .to_string()
        } else {
            "".to_string()
        };

        Bookmark {
            id: None,
            name,
            path: Some(abs_path),
            description,
        }
    }

    pub fn update(
        &mut self,
        name: Option<String>,
        path: Option<String>,
        description: Option<String>,
    ) {
        if name.is_some() {
            self.name = name;
        }
        if path.is_some() {
            self.path = path;
        }
        if description.is_some() {
            self.description = description;
        }
    }

    pub fn to_string(&self) -> String {
        let id = if self.id.is_none() {
            "None".to_string()
        } else {
            self.id.unwrap().to_string()
        };
        let name = self.name.clone().unwrap_or("None".to_string());
        let path = self.path.clone().unwrap_or("None".to_string());
        let description = self.description.clone().unwrap_or("None".to_string());
        format!(
            "id: {}, name: {}, path: {}, description: {}",
            id, name, path, description
        )
    }
}

impl Default for Bookmark {
    fn default() -> Self {
        Bookmark {
            id: None,
            name: None,
            path: None,
            description: None,
        }
    }
}
