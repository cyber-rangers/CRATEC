use rusqlite::{params, Connection, Result};

pub fn initialize_interface_scheme(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS interface (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            interface_path TEXT NOT NULL UNIQUE,
            side TEXT NOT NULL CHECK (side IN ('input','output')),
            name TEXT NOT NULL
        )",
        [],
    )?;

    let interfaces = [
        ("pci-0000:03:00.0-ata-1", "input",  "IN 1"),
        ("pci-0000:03:00.0-ata-2", "input",  "IN 2"),
        ("pci-0000:03:00.0-ata-3", "output", "OUT 1"),
        ("pci-0000:03:00.0-ata-4", "output", "OUT 2"),
        ("pci-0000:00:14.0-usb-0:1:1.0-scsi-0:0:0:0", "input",  "USB IN"),
        ("pci-0000:00:14.0-usb-0:2:1.0-scsi-0:0:0:0", "output", "USB OUT"),
    ];

    for (path, side, name) in interfaces {
        conn.execute(
            "INSERT OR IGNORE INTO interface (interface_path, side, name)
             VALUES (?1, ?2, ?3)",
            params![path, side, name],
        )?;
    }

    Ok(())
}

