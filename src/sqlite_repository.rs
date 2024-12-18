use rusqlite::Connection;


const DATABASE_PATH: &str = "/home/vlad/.config/rustmarks/rustmarks.db";

pub struct SqliteRepository {
    conn: Connection,
}

impl SqliteRepository {
    pub fn new() -> SqliteRepository {

        
        // Create the connection
        let conn = Connection::open(DATABASE_PATH).unwrap_or_else(|_| {

            // Create the database file if it doesn't exist
            
            std::fs::create_dir_all("/home/vlad/.config/rustmarks").unwrap();
            if !std::path::Path::new(DATABASE_PATH).exists() {
                std::fs::File::create(DATABASE_PATH).unwrap();
            }
            Connection::open(DATABASE_PATH).unwrap()    
        });

        // Create the table if it doesn't exist
        conn.execute(
            "CREATE TABLE IF NOT EXISTS bookmarks (
                id INTEGER PRIMARY KEY,
                name TEXT,
                path TEXT,
                description TEXT
            )",
            [],
        )
        .unwrap();

        SqliteRepository { conn }
    }
    pub fn list_bookmarks(&self) -> Result<Vec<Bookmark>, rusqlite::Error> {
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
    pub fn get_bookmark(&self, id: i32) -> Result<Bookmark, rusqlite::Error> {
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

    pub fn get_bookmark_by_path(&self, path: String) -> Result<Bookmark, rusqlite::Error> {
        let mut stmt = self.conn.prepare("SELECT * FROM bookmarks WHERE path = ?")?;
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
    pub fn get_bookmark_by_name(&self, name: String) -> Result<Bookmark, rusqlite::Error> {
        let mut stmt = self.conn.prepare("SELECT * FROM bookmarks WHERE name = ?")?;
        let bookmark = stmt.query_row(&[&name], |row| {
            Ok(Bookmark {
                id: row.get(0)?,
                name: row.get(1)?,
                path: row.get(2)?,
                description: row.get(3)?,
            })
        })?;
        Ok(bookmark)
    }
    pub fn add_bookmark(&self, bookmark: Bookmark) -> Result<(), rusqlite::Error> {
        
        // TODO: Check if a bookmark with the same path already exists
        let check_bookmark = self.get_bookmark_by_path(bookmark.path.clone().unwrap());
        if let Ok(bookmark) = check_bookmark {
            return Err(rusqlite::Error::QueryReturnedNoRows);
        }
        
        let mut stmt = self.conn.prepare("INSERT INTO bookmarks (name, path, description) VALUES (?, ?, ?)")?;
        stmt.execute(&[&bookmark.name, &bookmark.path, &bookmark.description])?;
        Ok(())
    }

    pub fn update_bookmark(&self, id: i32, bookmark: Bookmark) -> Result<(), rusqlite::Error> {
        let mut stmt = self.conn.prepare("UPDATE bookmarks SET name = ?, path = ?, description = ? WHERE id = ?")?;
        stmt.execute(&[&bookmark.name, &bookmark.path, &bookmark.description, &Option::from(id.to_string())])?;
        Ok(())
    }

    pub fn delete_bookmark(&self, id: i32) -> Result<(), rusqlite::Error> {
        let mut stmt = self.conn.prepare("DELETE FROM bookmarks WHERE id = ?")?;
        stmt.execute(&[&id])?;
        Ok(())
    }

}

pub struct Bookmark {
    pub id: i32,
    pub name: Option<String>,
    pub path: Option<String>,
    pub description: Option<String>,
}