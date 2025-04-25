use tauri::command;
use serde_json::{Value, Map, json};
use rusqlite::{params, Row, OptionalExtension};
use rusqlite::types::ValueRef;
use crate::db::DB_POOL;

/// Převod jednoho řádku na JSON objekt, používá předané názvy sloupců
fn row_to_json(row: &Row, col_names: &[String]) -> Value {
    let mut obj = Map::new();
    for (i, name) in col_names.iter().enumerate() {
        let v = match row.get_ref(i).unwrap() {
            ValueRef::Null => Value::Null,
            ValueRef::Integer(n) => Value::from(n),
            ValueRef::Real(f) => Value::from(f),
            ValueRef::Text(s) => Value::String(String::from_utf8_lossy(s).to_string()),
            ValueRef::Blob(b) => Value::String(base64::encode(b)),
        };
        obj.insert(name.clone(), v);
    }
    Value::Object(obj)
}

/// Pomocná funkce pro získání i64 z Value (string nebo číslo)
fn value_to_i64(val: &Value) -> Option<i64> {
    match val {
        Value::Number(n) => n.as_i64(),
        Value::String(s) => s.parse::<i64>().ok(),
        _ => None,
    }
}

/// Vrátí všechny záznamy copy_process a k nim odpovídající copy_log_ewf nebo copy_log_dd podle cizího klíče
#[command(rename_all = "snake_case")]
pub async fn get_history() -> Result<Value, String> {
    let mut pooled = DB_POOL.get_connection().map_err(|e| e.to_string())?;
    pooled.execute(|conn| {
        let mut stmt = conn.prepare("SELECT * FROM copy_process ORDER BY id DESC")?;
        let cols = stmt.column_names().iter().map(|s| s.to_string()).collect::<Vec<_>>();
        let mut items = Vec::new();

        let mut rows = stmt.query([])?;
        while let Some(r) = rows.next()? {
            let proc = row_to_json(r, &cols);

            // Získej hodnoty cizích klíčů jako Option<i64>
            let triggered_by_ewf = r.get::<_, Option<i64>>(cols.iter().position(|c| c == "triggered_by_ewf").unwrap()).ok().flatten();
            let triggered_by_dd  = r.get::<_, Option<i64>>(cols.iter().position(|c| c == "triggered_by_dd").unwrap()).ok().flatten();

            let mut copy_log = Value::Null;

            if let Some(eid) = triggered_by_ewf {
                let mut st = conn.prepare("SELECT * FROM copy_log_ewf WHERE id = ?1")?;
                let log_cols = st.column_names().iter().map(|s| s.to_string()).collect::<Vec<_>>();
                if let Some(log) = st.query_row(params![eid], |r| Ok(row_to_json(r, &log_cols))).optional()? {
                    copy_log = log;
                }
            } else if let Some(did) = triggered_by_dd {
                let mut st = conn.prepare("SELECT * FROM copy_log_dd WHERE id = ?1")?;
                let log_cols = st.column_names().iter().map(|s| s.to_string()).collect::<Vec<_>>();
                if let Some(log) = st.query_row(params![did], |r| Ok(row_to_json(r, &log_cols))).optional()? {
                    copy_log = log;
                }
            }

            items.push(json!({
                "process": proc,
                "copy_log": copy_log
            }));
        }
        
        Ok(Value::Array(items))
    }).map_err(|e| e.to_string())
}


/// Vrátí pole textových řádků logu pro daný process_id
#[command(rename_all = "snake_case")]
pub async fn get_process_log_lines_texts(process_id: i64) -> Result<Value, String> {
    let mut pooled = DB_POOL.get_connection().map_err(|e| e.to_string())?;
    pooled.execute(|conn| {
        let mut st = conn.prepare(
            "SELECT line_content FROM process_log_lines WHERE process_id = ?1 ORDER BY line_number ASC"
        )?;
        let mut lines = Vec::new();
        let mut rows = st.query(params![process_id])?;
        while let Some(r) = rows.next()? {
            match r.get_ref(0)? {
                ValueRef::Text(s) => lines.push(Value::String(String::from_utf8_lossy(s).to_string())),
                ValueRef::Blob(b) => {
                    match std::str::from_utf8(b) {
                        Ok(txt) => lines.push(Value::String(txt.to_string())),
                        Err(_) => lines.push(Value::String(base64::encode(b))),
                    }
                },
                ValueRef::Null => lines.push(Value::Null),
                _ => lines.push(Value::String("<nepodporovaný typ>".to_string())),
            }
        }
        if lines.is_empty() {
            Ok(Value::String("Není dostupný žádný výpis".to_string()))
        } else {
            Ok(Value::Array(lines))
        }
    }).map_err(|e| {
        println!("get_process_log_lines_texts error: {}", e);
        e.to_string()
    })
}

/// 3) Vrátí konfiguraci z `ewf_config` nebo `dd_config` podle `config_type`
#[command(rename_all = "snake_case")]
pub async fn get_config_entry(config_id: i64, config_type: String) -> Result<Value, String> {
    let table = match config_type.as_str() {
        "ewf" | "ewf_config" => "ewf_config",
        "dd"  | "dd_config"  => "dd_config",
        other => return Err(format!("Unknown config_type: {}", other))
    };
    let sql = format!("SELECT * FROM {} WHERE id = ?1 AND active = 1", table);
    let mut pooled = DB_POOL.get_connection().map_err(|e| e.to_string())?;
    pooled.execute(|conn| {
        let mut st = conn.prepare(&sql)?;
        let cols = st.column_names().iter().map(|s| s.to_string()).collect::<Vec<_>>();
        if let Some(r) = st.query_row(params![config_id], |r| Ok(row_to_json(r, &cols))).optional()? {
            Ok(r)
        } else {
            Ok(Value::Null)
        }
    }).map_err(|e| e.to_string())
}