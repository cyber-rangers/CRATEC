use serde::{Serialize, Deserialize};
use serde_json::{Map, Value, json};
use rusqlite::{Connection, Row, Statement, params};
use std::error::Error;

/// Pomocná funkce, která z jedné řádky výsledku SQL a příslušného Statementu 
/// vygeneruje dynamický JSON objekt se všemi sloupci.
fn row_to_json(row: &Row, stmt: &Statement) -> serde_json::Value {
    let mut obj = Map::new();
    // Projdeme všechny sloupce v dotazu
    for (i, name) in stmt.column_names().iter().enumerate() {
        // Zkusíme získat hodnotu jako String (v jedné DB mohou být různá datová pole,
        // ale pro ukázku budeme vše načítat jako text, příp. NATURAL je potřeba sladit s Vaším schématem)
        let val: rusqlite::Result<String> = row.get(i);
        let value: Value = match val {
            Ok(s) => Value::String(s),
            Err(_) => Value::Null,
        };
        obj.insert(name.to_string(), value);
    }
    Value::Object(obj)
}

/// Funkce, která dotáže `copy_process` a vrátí pole JSON objektů.
/// Každý objekt bude obsahovat:
/// - "process": celý záznam z `copy_process`
/// - "logs": všechny řádky z `process_log_lines`
/// - "ewf_log": pokud `triggered_by_ewf` není NULL, tak záznam z `copy_log_ewf`
/// - "dd_log": pokud `triggered_by_dd` není NULL, tak záznam z `copy_log_dd`
pub fn get_acquisition_history_json(conn: &Connection) -> Result<String, Box<dyn Error>> {
    // 1) Dotaz na všechna pole z tabulky copy_process
    let mut stmt_proc = conn.prepare("SELECT * FROM copy_process ORDER BY id DESC")?;
    let rows = stmt_proc.query_map([], |row| {
        // Převést row na dynamický JSON (všechna pole)
        Ok(row_to_json(row, &stmt_proc))
    })?;

    let mut results = vec![];

    // 2) Pro každý řádek copy_process vyhledat související záznamy
    for process_value in rows {
        let process_json = process_value?;

        // process_id potřebujeme k vyhledání logů
        // Přestože row_to_json z `copy_process` vrátil dynamický JSON, 
        // musíme zajistit, že 'id' bylo text/číslo. Pokud je v DB integer, 
        // bude vrácen jako string, proto ho zkusíme převést.
        let process_id_opt = process_json
            .get("id")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<i64>().ok());

        // Získáme triggered_by_ewf a triggered_by_dd
        let triggered_by_ewf_opt = process_json
            .get("triggered_by_ewf")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<i64>().ok());

        let triggered_by_dd_opt = process_json
            .get("triggered_by_dd")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<i64>().ok());

        // 3) Vyhledáme logy z process_log_lines
        let mut logs_array = vec![];
        if let Some(process_id) = process_id_opt {
            let mut stmt_logs = conn.prepare("SELECT * FROM process_log_lines WHERE process_id = ? ORDER BY line_number ASC")?;
            let log_iter = stmt_logs.query_map(params![process_id], |row| {
                Ok(row_to_json(row, &stmt_logs))
            })?;

            for log_json in log_iter {
                logs_array.push(log_json?);
            }
        }

        // 4) Pokud je triggered_by_ewf, najdeme záznam v `copy_log_ewf`
        let mut ewf_log: Option<Value> = None;
        if let Some(eid) = triggered_by_ewf_opt {
            let mut stmt_ewf = conn.prepare("SELECT * FROM copy_log_ewf WHERE id = ?")?;
            let ewf_row = stmt_ewf.query_map(params![eid], |row| {
                Ok(row_to_json(row, &stmt_ewf))
            })?.next();

            if let Some(Ok(val)) = ewf_row {
                ewf_log = Some(val);
            }
        }

        // 5) Pokud je triggered_by_dd, najdeme záznam v `copy_log_dd`
        let mut dd_log: Option<Value> = None;
        if let Some(did) = triggered_by_dd_opt {
            let mut stmt_dd = conn.prepare("SELECT * FROM copy_log_dd WHERE id = ?")?;
            let dd_row = stmt_dd.query_map(params![did], |row| {
                Ok(row_to_json(row, &stmt_dd))
            })?.next();

            if let Some(Ok(val)) = dd_row {
                dd_log = Some(val);
            }
        }

        // Sestavíme výsledný objekt pro daný záznam
        let item = json!({
            "process": process_json,
            "logs": logs_array,
            "ewf_log": ewf_log.unwrap_or(Value::Null),
            "dd_log": dd_log.unwrap_or(Value::Null),
        });
        results.push(item);
    }

    // 6) Výsledky jako JSON pole (pretty print)
    let json_array = serde_json::to_string_pretty(&results)?;
    Ok(json_array)
}