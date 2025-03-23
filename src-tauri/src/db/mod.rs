use rusqlite::Connection;
use std::error::Error;
use std::fs;
use std::path::Path;

pub mod logging_scheme;
pub mod ewf_config_scheme;
pub mod copy_log_scheme;
pub mod dd_config_scheme;
pub mod interface_scheme;
pub mod process_log_scheme;


pub fn initialize_db() -> Result<Connection, Box<dyn Error>> {
    let db_path = "/var/lib/cratec/database.db";

    if let Some(parent_dir) = Path::new(db_path).parent() {
        fs::create_dir_all(parent_dir)?;
    }

    let conn = Connection::open(db_path)?;

    logging_scheme::initialize_logging_scheme(&conn)?;
    ewf_config_scheme::initialize_ewf_config_scheme(&conn)?;
    dd_config_scheme::initialize_dd_config_scheme(&conn)?;
    copy_log_scheme::initialize_copy_log_scheme(&conn)?;
    interface_scheme::initialize_interface_scheme(&conn)?;
    process_log_scheme::initialize_process_log_scheme(&conn)?;

    Ok(conn)
}


pub fn connect_db() -> Result<Connection, Box<dyn Error>> {
    let db_path = "/var/lib/cratec/database.db";
    let conn = Connection::open(db_path)?;
    Ok(conn)
}
