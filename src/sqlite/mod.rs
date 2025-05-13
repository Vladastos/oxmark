use rusqlite::Connection;

use super::Bookmark;

const DATABASE_DIR: &str = ".local/share/oxmark";
const DATABASE_NAME: &str = "oxmark.db";

/*
 *
 * SQLite Service
 *
 */

pub struct SqliteService {
    sqlite_repository: SqliteRepository,
}

impl SqliteService {
    pub fn new() -> Result<SqliteService, SqliteServiceError> {
        let result = SqliteRepository::new();

        if let Err(_) = result {
            return Err(SqliteServiceError::InternalError);
        }
        Ok(SqliteService {
            sqlite_repository: result.unwrap(),
        })
    }
    pub fn create(
        &self,
        path: String,
        name: Option<String>,
        description: Option<String>,
    ) -> Result<(), SqliteServiceError> {
        let bookmark = Bookmark::new(name, Some(path), description);

        // Check if a bookmark with the same path already exists
        let check_bookmark = self
            .sqlite_repository
            .get_bookmark_by_path(bookmark.path.clone().unwrap());
        if let Ok(_) = check_bookmark {
            return Err(SqliteServiceError::BookmarkAlreadyExists);
        }
        let result = self.sqlite_repository.create_bookmark(bookmark);
        if let Err(_) = result {
            return Err(SqliteServiceError::InternalError);
        }
        Ok(())
    }
    pub fn delete(&self, path: String) -> Result<(), SqliteServiceError> {
        // TODO: handle canonicalize errors

        let abs_path = std::path::PathBuf::from(path.clone())
            .canonicalize()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();

        let bookmark_result = self.sqlite_repository.get_bookmark_by_path(abs_path);

        if let Err(_) = bookmark_result {
            return Err(SqliteServiceError::PathNotFound(path));
        }

        let bookmark = bookmark_result.unwrap();

        let result = self.sqlite_repository.delete_bookmark(bookmark.id.unwrap());

        if let Err(_) = result {
            return Err(SqliteServiceError::InternalError);
        }

        Ok(())
    }

    pub fn update(
        &self,
        id: i32,
        mut path: Option<String>,
        name: Option<String>,
        description: Option<String>,
    ) -> Result<(), SqliteServiceError> {
        // If the path is provided, canonicalize it and get the bookmark by path
        if path.is_some() {
            // TODO: handle canonicalize errors

            path = Option::from(
                std::path::PathBuf::from(path.clone().unwrap())
                    .canonicalize()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_string(),
            );

            let test_bookmark = self
                .sqlite_repository
                .get_bookmark_by_path(path.clone().unwrap());

            // If the bookmark already exists, return an error
            if let Ok(_) = test_bookmark {
                return Err(SqliteServiceError::BookmarkAlreadyExists);
            }
        }

        //  Get the bookmark by id
        let result = self.sqlite_repository.get_bookmark(id);
        if let Err(_) = result {
            return Err(SqliteServiceError::IdNotFound(id));
        }

        let mut bookmark = result.unwrap();

        // Update the bookmark
        bookmark.update(name, path, description);

        let result = self.sqlite_repository.update_bookmark(id, bookmark.clone());

        if let Err(_) = result {
            return Err(SqliteServiceError::InternalError);
        }

        Ok(())
    }

    pub fn get_all(&self, pathsonly: bool) -> Result<(), SqliteServiceError> {
        let bookmarks = self
            .sqlite_repository
            .get_all_bookmarks()
            .unwrap_or_else(|e| {
                println!("{}", e);
                std::process::exit(1);
            });

        // If pathsonly is true, print only the paths
        if pathsonly {
            bookmarks.iter().for_each(|bookmark| {
                println!("{}", bookmark.path.clone().unwrap());
            });

            return Ok(());
        }

        // Otherwise, print all the bookmarks
        bookmarks.iter().for_each(|bookmark| {
            println!("{}", bookmark.to_string());
        });

        Ok(())
    }
}

//
// SqliteServiceError
//

#[derive(Debug)]
pub enum SqliteServiceError {
    IdNotFound(i32),
    PathNotFound(String),
    BookmarkAlreadyExists,
    InternalError,
}

impl SqliteServiceError {
    pub fn message(&self) -> String {
        match self {
            SqliteServiceError::PathNotFound(path) => {
                format!("Bookmark with path {} not found", path)
            }
            SqliteServiceError::IdNotFound(id) => format!("Bookmark with id {} not found", id),
            SqliteServiceError::BookmarkAlreadyExists => "Bookmark already exists".to_string(),
            SqliteServiceError::InternalError => "Internal error".to_string(),
        }
    }
}

impl std::fmt::Display for SqliteServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "oxmark - SqliteServiceError: {}", self.message())
    }
}
impl std::error::Error for SqliteServiceError {}

/*
 *
 * SQLite Repository
 *
 */

#[derive(Debug)]
pub struct SqliteRepository {
    conn: Connection,
}

impl SqliteRepository {
    pub fn new() -> Result<SqliteRepository, SqliteRepositoryError> {
        // Get the $HOME environment variable
        let home_dir = std::env::var("HOME").unwrap();
        let db_dir_path = std::path::Path::new(&home_dir).join(DATABASE_DIR);
        let db_path = std::path::Path::new(&db_dir_path).join(DATABASE_NAME);

        let mut result = Connection::open(db_path.clone());

        // If the connection failed, try to create the database file
        if let Err(_) = result {
            std::fs::create_dir_all(db_dir_path).unwrap();
            if !std::path::Path::new(&db_path).exists() {
                let file_creation_result = std::fs::File::create(db_path.clone());
                if let Err(e) = file_creation_result {
                    return Err(SqliteRepositoryError::IoError(e));
                }

                result = Connection::open(db_path);
            }
        }

        // If the connection failed, return an error
        if let Err(_) = result {
            return Err(SqliteRepositoryError::ConnectionError);
        }

        let conn = result.unwrap();

        // Create the table if it doesn't exist
        let query_result = conn.execute(
            "CREATE TABLE IF NOT EXISTS bookmarks (
                id INTEGER PRIMARY KEY,
                name TEXT,
                path TEXT,
                description TEXT
            )",
            [],
        );

        if let Err(e) = query_result {
            return Err(SqliteRepositoryError::QueryError(e));
        }

        Ok(SqliteRepository { conn })
    }
    pub fn get_all_bookmarks(&self) -> Result<Vec<Bookmark>, SqliteRepositoryError> {
        let mut bookmarks_vec: Vec<Bookmark> = Vec::new();
        let mut stmt = self.conn.prepare("SELECT * FROM bookmarks")?;
        let bookmarks = stmt.query_map([], |row| {
            Ok(Bookmark {
                id: row.get(0)?,
                name: row.get(1)?,
                path: row.get(2)?,
                description: row.get(3)?,
            })
        })?;
        for bookmark in bookmarks {
            bookmarks_vec.push(bookmark.unwrap());
        }
        Ok(bookmarks_vec)
    }
    pub fn get_bookmark(&self, id: i32) -> Result<Bookmark, SqliteRepositoryError> {
        let mut stmt = self.conn.prepare("SELECT * FROM bookmarks WHERE id = ?")?;
        let bookmark = stmt.query_row(&[&id], |row| {
            Ok(Bookmark {
                id: row.get(0)?,
                name: row.get(1)?,
                path: row.get(2)?,
                description: row.get(3)?,
            })
        })?;
        Ok(bookmark)
    }

    pub fn get_bookmark_by_path(&self, path: String) -> Result<Bookmark, SqliteRepositoryError> {
        let mut stmt = self
            .conn
            .prepare("SELECT * FROM bookmarks WHERE path = ?")?;
        let bookmark = stmt.query_row(&[&path], |row| {
            Ok(Bookmark {
                id: row.get(0)?,
                name: row.get(1)?,
                path: row.get(2)?,
                description: row.get(3)?,
            })
        })?;
        Ok(bookmark)
    }

    pub fn create_bookmark(&self, bookmark: Bookmark) -> Result<(), SqliteRepositoryError> {
        let mut stmt = self
            .conn
            .prepare("INSERT INTO bookmarks (name, path, description) VALUES (?, ?, ?)")?;
        stmt.execute(&[&bookmark.name, &bookmark.path, &bookmark.description])?;
        Ok(())
    }

    pub fn update_bookmark(
        &self,
        id: i32,
        bookmark: Bookmark,
    ) -> Result<(), SqliteRepositoryError> {
        let mut stmt = self
            .conn
            .prepare("UPDATE bookmarks SET name = ?, path = ?, description = ? WHERE id = ?")?;
        stmt.execute(&[
            &bookmark.name,
            &bookmark.path,
            &bookmark.description,
            &Option::from(id.to_string()),
        ])?;
        Ok(())
    }

    pub fn delete_bookmark(&self, id: i32) -> Result<(), SqliteRepositoryError> {
        let mut stmt = self.conn.prepare("DELETE FROM bookmarks WHERE id = ?")?;
        stmt.execute(&[&id])?;
        Ok(())
    }
}

//
// SqliteRepositoryError
//

#[derive(Debug)]
pub enum SqliteRepositoryError {
    NotFound,
    InternalError(rusqlite::Error),
    ConnectionError,
    IoError(std::io::Error),
    QueryError(rusqlite::Error),
}

impl SqliteRepositoryError {
    pub fn message(&self) -> String {
        match self {
            SqliteRepositoryError::NotFound => "Bookmark not found".to_string(),
            SqliteRepositoryError::InternalError(e) => format!("Internal error: {}", e),
            SqliteRepositoryError::ConnectionError => "Connection error".to_string(),
            SqliteRepositoryError::IoError(e) => format!("IO error: {}", e),
            SqliteRepositoryError::QueryError(e) => format!("Query error: {}", e),
        }
    }
}

impl std::fmt::Display for SqliteRepositoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "oxmark - SqliteRepositoryError: {}", self.message())
    }
}

impl From<rusqlite::Error> for SqliteRepositoryError {
    fn from(e: rusqlite::Error) -> Self {
        match e {
            rusqlite::Error::QueryReturnedNoRows => SqliteRepositoryError::NotFound,
            _ => SqliteRepositoryError::InternalError(e),
        }
    }
}

impl std::error::Error for SqliteRepositoryError {}
