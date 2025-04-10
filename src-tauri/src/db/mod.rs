use rusqlite::{Connection, OpenFlags};
use std::error::Error;
use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex as StdMutex};
use once_cell::sync::Lazy;
use std::collections::VecDeque;
use std::time::{Instant, Duration, SystemTime, UNIX_EPOCH};
use std::thread;

pub mod copy_log_scheme;
pub mod dd_config_scheme;
pub mod ewf_config_scheme;
pub mod interface_scheme;
pub mod logging_scheme;
pub mod process_log_scheme;

// Maximum number of connections in the pool
const MAX_POOL_SIZE: usize = 10;
// Number of retries for getting a connection
const MAX_RETRIES: usize = 5;
// Time to wait between retries (milliseconds)
const RETRY_WAIT_MS: u64 = 100;

// Database path
const DB_PATH: &str = "/var/lib/cratec/database.db";

// Counter for tracking connection IDs
static mut CONNECTION_COUNTER: u64 = 0;

// Simple function to generate a unique connection ID
fn get_next_connection_id() -> u64 {
    unsafe {
        CONNECTION_COUNTER += 1;
        CONNECTION_COUNTER
    }
}

// Helper function to get current timestamp for logging
fn get_timestamp() -> String {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    format!("{}", timestamp)
}

// Database connection wrapper with automatic cleanup
struct DatabaseConnection {
    conn: Connection,
    id: u64,
}

impl Drop for DatabaseConnection {
    fn drop(&mut self) {
        println!("[{}] [DB:{}] Connection dropped", get_timestamp(), self.id);
    }
}

/// Type for the connection pool
pub struct ConnectionPool {
    connections: StdMutex<VecDeque<DatabaseConnection>>,
}

impl ConnectionPool {
    // Create a new connection pool with initial connections
    fn new(size: usize) -> Result<Self, Box<dyn Error>> {
        println!("[{}] Creating connection pool with initial size {}", get_timestamp(), size);
        let mut connections = VecDeque::with_capacity(size);
        
        // Create initial connections
        for _ in 0..size {
            connections.push_back(Self::create_connection()?);
        }
        
        println!("[{}] Connection pool initialized successfully with {} connections", get_timestamp(), size);
        Ok(Self {
            connections: StdMutex::new(connections),
        })
    }
    
    // Create a single database connection
    fn create_connection() -> Result<DatabaseConnection, Box<dyn Error>> {
        let start_time = Instant::now();
        let connection_id = get_next_connection_id();
        
        println!("[{}] [DB:{}] Creating new database connection...", get_timestamp(), connection_id);
        let conn = Connection::open_with_flags(
            DB_PATH,
            OpenFlags::SQLITE_OPEN_READ_WRITE |
            OpenFlags::SQLITE_OPEN_CREATE |
            OpenFlags::SQLITE_OPEN_FULL_MUTEX
        )?;
        
        // Configure connection
        conn.pragma_update(None, "journal_mode", &"WAL")?;
        conn.busy_timeout(Duration::from_secs(30))?;
        conn.pragma_update(None, "synchronous", &"NORMAL")?;
        conn.pragma_update(None, "cache_size", &"10000")?;
        conn.pragma_update(None, "locking_mode", &"NORMAL")?;
        
        conn.execute_batch(
            "PRAGMA foreign_keys = ON;
             PRAGMA temp_store = MEMORY;"
        )?;
        
        let elapsed = start_time.elapsed().as_millis();
        println!("[{}] [DB:{}] Connection created successfully in {} ms", get_timestamp(), connection_id, elapsed);
        
        Ok(DatabaseConnection {
            conn,
            id: connection_id,
        })
    }
    
    // Get a connection from the pool or create a new one if needed
    pub fn get_connection(&self) -> Result<PooledConnection, Box<dyn Error>> {
        for attempt in 0..MAX_RETRIES {
            // Try to get a connection from the pool
            let mut guard = match self.connections.try_lock() {
                Ok(guard) => guard,
                Err(_) => {
                    println!("[{}] Failed to acquire pool lock, attempt {}/{}", get_timestamp(), attempt + 1, MAX_RETRIES);
                    if attempt + 1 < MAX_RETRIES {
                        thread::sleep(Duration::from_millis(RETRY_WAIT_MS));
                        continue;
                    }
                    return Err("Failed to acquire connection pool lock after retries".into());
                }
            };
            
            if let Some(conn) = guard.pop_front() {
                println!("[{}] [DB:{}] Obtained connection from pool", get_timestamp(), conn.id);
                return Ok(PooledConnection {
                    conn: Some(conn),
                    pool: self,
                });
            } else if guard.len() < MAX_POOL_SIZE {
                // If pool is not at max capacity, create a new connection
                drop(guard); // Release lock before creating connection
                let conn = Self::create_connection()?;
                let conn_id = conn.id;
                return Ok(PooledConnection {
                    conn: Some(conn),
                    pool: self,
                });
            }
            
            // If pool is at capacity and no connections available, wait and retry
            drop(guard);
            println!("[{}] No connections available in pool, waiting... (attempt {}/{})", 
                get_timestamp(), attempt + 1, MAX_RETRIES);
            thread::sleep(Duration::from_millis(RETRY_WAIT_MS));
        }
        
        // Create a new connection if we couldn't get one from the pool after retries
        println!("[{}] Creating new connection outside pool after retries", get_timestamp());
        let conn = Self::create_connection()?;
        let conn_id = conn.id;
        Ok(PooledConnection {
            conn: Some(conn),
            pool: self,
        })
    }
    
    // Return a connection to the pool
    fn return_connection(&self, conn: DatabaseConnection) {
        if let Ok(mut guard) = self.connections.try_lock() {
            if guard.len() < MAX_POOL_SIZE {
                println!("[{}] [DB:{}] Returning connection to pool", get_timestamp(), conn.id);
                guard.push_back(conn);
                return;
            }
        }
        
        // If pool is full or locked, let the connection drop
        println!("[{}] [DB:{}] Connection dropped (pool full or locked)", get_timestamp(), conn.id);
    }
}

// Pooled connection wrapper that returns connection to pool on drop
pub struct PooledConnection<'a> {
    conn: Option<DatabaseConnection>,
    pool: &'a ConnectionPool,
}

impl<'a> PooledConnection<'a> {
    // Get the underlying rusqlite connection
    pub fn connection(&mut self) -> &mut Connection {
        &mut self.conn.as_mut().unwrap().conn
    }
    
    // Execute a closure with the connection
    pub fn execute<F, T>(&mut self, f: F) -> Result<T, Box<dyn Error>>
    where
        F: FnOnce(&mut Connection) -> Result<T, Box<dyn Error>>,
    {
        let start = Instant::now();
        let conn_id = self.conn.as_ref().unwrap().id;
        println!("[{}] [DB:{}] Executing database operation", get_timestamp(), conn_id);
        
        let result = f(self.connection())?;
        
        let elapsed = start.elapsed().as_millis();
        println!("[{}] [DB:{}] Database operation completed in {} ms", get_timestamp(), conn_id, elapsed);
        
        Ok(result)
    }
}

impl<'a> Drop for PooledConnection<'a> {
    fn drop(&mut self) {
        if let Some(conn) = self.conn.take() {
            self.pool.return_connection(conn);
        }
    }
}

// Global connection pool
pub static DB_POOL: Lazy<Arc<ConnectionPool>> = Lazy::new(|| {
    println!("[{}] Initializing global DB connection pool...", get_timestamp());
    let pool = ConnectionPool::new(5).expect("Failed to initialize DB connection pool");
    println!("[{}] Global DB connection pool initialized successfully", get_timestamp());
    Arc::new(pool)
});

/// Inicializuje databázi – vytvoří tabulky a další schémata.
pub fn initialize_db() -> Result<(), Box<dyn Error>> {
    println!("[{}] Initializing database schema...", get_timestamp());
    
    // Ujisti se, že existuje adresář pro databázi.
    if let Some(parent_dir) = Path::new(DB_PATH).parent() {
        fs::create_dir_all(parent_dir)?;
    }
    
    // Get a connection from the pool and initialize schemas
    let mut conn = DB_POOL.get_connection()?;
    
    conn.execute(|conn| {
        // Inicializace schémat v databázi.
        logging_scheme::initialize_logging_scheme(conn)?;
        ewf_config_scheme::initialize_ewf_config_scheme(conn)?;
        dd_config_scheme::initialize_dd_config_scheme(conn)?;
        copy_log_scheme::initialize_copy_log_scheme(conn)?;
        interface_scheme::initialize_interface_scheme(conn)?;
        process_log_scheme::initialize_process_log_scheme(conn)?;
        
        Ok(())
    })?;
    
    println!("[{}] Database schema initialized successfully", get_timestamp());
    Ok(())
}

// For compatibility with existing code
pub type SharedConnection = Arc<tokio::sync::Mutex<Connection>>;

// Static DB_CONN kept for backwards compatibility
pub static DB_CONN: Lazy<SharedConnection> = Lazy::new(|| {
    println!("[{}] Creating legacy DB_CONN (deprecated, use DB_POOL instead)...", get_timestamp());
    let conn = create_new_connection().expect("Failed to initialize legacy DB connection");
    println!("[{}] Legacy DB_CONN created", get_timestamp());
    Arc::new(tokio::sync::Mutex::new(conn))
});

/// Create a new standalone database connection (prefer using the pool)
pub fn create_new_connection() -> Result<Connection, Box<dyn Error>> {
    let start_time = Instant::now();
    let connection_id = get_next_connection_id();
    
    println!("[{}] [DB:{}] Creating standalone connection (not pooled)...", get_timestamp(), connection_id);
    
    let conn = Connection::open_with_flags(
        DB_PATH,
        OpenFlags::SQLITE_OPEN_READ_WRITE |
        OpenFlags::SQLITE_OPEN_CREATE |
        OpenFlags::SQLITE_OPEN_FULL_MUTEX
    )?;
    
    // Configure connection
    conn.pragma_update(None, "journal_mode", &"WAL")?;
    conn.busy_timeout(Duration::from_secs(60))?; // Increase timeout to 60 seconds
    conn.pragma_update(None, "synchronous", &"NORMAL")?;
    conn.pragma_update(None, "cache_size", &"10000")?;
    conn.pragma_update(None, "locking_mode", &"NORMAL")?;
    
    // Add additional pragmas to optimize performance
    conn.pragma_update(None, "temp_store", &"MEMORY")?;
    conn.pragma_update(None, "mmap_size", &(64 * 1024 * 1024))?; // 64MB mmap
    
    // Set this connection to wait longer for locks
    conn.pragma_update(None, "busy_timeout", &5000)?;
    
    let elapsed = start_time.elapsed().as_millis();
    println!("[{}] [DB:{}] Standalone connection created in {} ms", get_timestamp(), connection_id, elapsed);
    
    Ok(conn)
}

/// Execute SQL statement with retries for database locked errors
pub fn execute_sql_with_retries(
    conn: &mut Connection, 
    sql: &str, 
    params: &[&dyn rusqlite::ToSql], 
    max_retries: usize
) -> Result<usize, Box<dyn Error>> {
    let mut retries = 0;
    let mut last_error = None;
    
    // Create a simple identifier for the SQL statement without using md5
    let sql_preview = if sql.len() > 20 {
        format!("{}...", &sql[0..20])
    } else {
        sql.to_string()
    };
    
    println!("[{}] Executing SQL (preview: {}), max_retries={}", 
        get_timestamp(), sql_preview, max_retries);
    
    while retries < max_retries {
        match conn.execute(sql, params) {
            Ok(rows) => {
                if retries > 0 {
                    println!("[{}] SQL (preview: {}) succeeded after {} retries", 
                        get_timestamp(), sql_preview, retries);
                }
                return Ok(rows);
            },
            Err(e) => {
                retries += 1;
                last_error = Some(e);
                
                if let rusqlite::Error::SqliteFailure(error, _) = &last_error.as_ref().unwrap() {
                    if error.code == rusqlite::ErrorCode::DatabaseBusy || 
                       error.code == rusqlite::ErrorCode::DatabaseLocked {
                        let wait_ms = 100 * (1 << retries); // Exponential backoff
                        println!("[{}] Database locked on SQL (preview: {}), retry {}/{} after {}ms", 
                            get_timestamp(), sql_preview, retries, max_retries, wait_ms);
                        std::thread::sleep(Duration::from_millis(wait_ms));
                        continue;
                    }
                }
                
                break;
            }
        }
    }
    
    Err(Box::new(last_error.unwrap()))
}
