use std::fs;
use tectonic;
use tera::{Context, Tera};

static TEMPLATE: &str = include_str!("./templates/en.tex");

/// Vygeneruje report pomocí mock dat – původní implementace.
pub fn mock_data_report() -> Result<(), String> {
    println!("📄 Vkládám šablonu přímo z kódu...");

    let mut context = Context::new();

    // Nastavení proměnných pro šablonu
    context.insert("software_hash", "75f1c14d734ea09147330fae210faa54");
    context.insert("build_date", "Jul 08, 2024 13:38:46 PDT");
    context.insert("serial_number", "117495");
    context.insert("time_local", "12:17:32 (CEST +0200)");
    context.insert("date", "Jul 08, 2024");
    context.insert("mode", "DriveToFile");
    context.insert("method", "DDCapture");
    context.insert("hash_type", "SHA-1");
    context.insert("image_path", "/var/reports/.../Kingston480GB\\_SATA");
    context.insert("lba_count", &937703088u64);
    context.insert("sector_size", &512u64);
    context.insert("segment_size", "WholeDisk");
    context.insert("compression", "None");
    context.insert("hash_enabled", &true);
    context.insert("verify_hash", &false);
    context.insert("unlock_hpa", &true);
    context.insert("unlock_dco", &true);
    context.insert("granularity", "SUCCESS");
    context.insert("result", "SUCCESS");
    context.insert("duration", "00:47:20");
    context.insert("time_complete", "12:14:52");
    context.insert("hash_lba_count", &937703088u64);
    context.insert("hash_sector_size", &512u64);
    context.insert("hash_primary_type", "SHA-1");
    context.insert("hash_primary", "75f1c14d734ea09147330fae210faa54");
    context.insert("hash_secondary_type", "MD5");
    context.insert("hash_secondary", "abcdef1234567890abcdef1234567890");
    context.insert("case_file", "CaseFile\\_001");
    context.insert("case_id", "CASE-2024-001");
    context.insert("examiner", "John Doe");
    context.insert("notes", "This case involves encrypted drives.");
    context.insert("segment_uid", "SEG123456");
    context.insert("segment_path", "/segments/seg123456");
    context.insert("segment_fs", "NTFS");
    context.insert("segment_serial", "SEG-SERIAL-001");
    context.insert("segment_file", "segment\\_file.img");
    context.insert("segment_hash", "seg\\_hash\\_abc123");

    // Ukázka disků
    let drives = vec![
        {
            let mut map = std::collections::HashMap::new();
            map.insert("bay", "1");
            map.insert("role", "Primary");
            map.insert("serial", "DRIVE123456");
            map.insert("model", "WD Blue");
            map.insert("raid", "RAID0");
            map.insert("fs", "EXT4");
            map.insert("cipher", "None");
            map
        },
        {
            let mut map = std::collections::HashMap::new();
            map.insert("bay", "2");
            map.insert("role", "Secondary");
            map.insert("serial", "DRIVE654321");
            map.insert("model", "Seagate Barracuda");
            map.insert("raid", "RAID1");
            map.insert("fs", "EXT4");
            map.insert("cipher", "AES-256");
            map
        },
    ];
    context.insert("drives", &drives);

    // Kapacity
    let capacities = vec![
        {
            let mut map = std::collections::HashMap::new();
            map.insert("bay", "1");
            map.insert("serial", "DRIVE123456");
            map.insert("capacity", "500");
            map
        },
        {
            let mut map = std::collections::HashMap::new();
            map.insert("bay", "2");
            map.insert("serial", "DRIVE654321");
            map.insert("capacity", "1000");
            map
        },
    ];
    context.insert("capacities", &capacities);

    // ATA security
    let ata_security = vec![{
        let mut map = std::collections::HashMap::new();
        map.insert("bay", "1");
        map.insert("role", "Primary");
        map.insert("enabled", "Yes");
        map.insert("locked", "No");
        map
    }];
    context.insert("ata_security", &ata_security);

    // Encryption
    let encryption = vec![{
        let mut map = std::collections::HashMap::new();
        map.insert("bay", "2");
        map.insert("role", "Secondary");
        map.insert("encrypted", "Yes");
        map.insert("locked", "Yes");
        map
    }];
    context.insert("encryption", &encryption);

    // Partitions
    let partitions = vec![
        {
            let mut map = std::collections::HashMap::new();
            map.insert("index", "1");
            map.insert("fs", "NTFS");
            map.insert("start", "2048");
            map.insert("end", "409600");
            map.insert("size", "200MB");
            map.insert("encryption", "Yes");
            map.insert("decrypted", "No");
            map
        },
        {
            let mut map = std::collections::HashMap::new();
            map.insert("index", "2");
            map.insert("fs", "FAT32");
            map.insert("start", "409601");
            map.insert("end", "819200");
            map.insert("size", "200MB");
            map.insert("encryption", "No");
            map.insert("decrypted", "Yes");
            map
        },
    ];
    context.insert("partitions", &partitions);

    println!("🧾 Vyrenderuji šablonu z paměti...");
    let latex_code = Tera::one_off(TEMPLATE, &context, false)
        .map_err(|e| format!("Chyba při renderování šablony: {}", e))?;

    fs::write("/home/master/Dokumenty/debug_output.tex", &latex_code)
        .map_err(|e| format!("Nelze uložit debug_output.tex: {}", e))?;

    println!("🧾 Kompiluji PDF pomocí Tectonic...");
    match tectonic::latex_to_pdf(&latex_code) {
        Ok(pdf_data) => {
            fs::write("/home/master/Dokumenty/output.pdf", pdf_data)
                .map_err(|e| format!("Nelze uložit output.pdf: {}", e))?;
            println!("✅ PDF úspěšně vytvořeno: output.pdf");
        }
        Err(e) => {
            eprintln!("❌ Chyba při kompilaci: {:#?}", e);
            println!("🧪 LaTeX byl uložen do debug_output.tex.");
            return Err(format!("Chyba při kompilaci: {:?}", e));
        }
    }

    Ok(())
}

/// Nová funkce generate_report, která zatím přijímá copy_process id a nedělá nic dalšího.
pub fn generate_report(copy_process_id: i64) -> Result<(), String> {
    println!("Generuji report pro copy_process id: {}", copy_process_id);

    let mut pooled_conn = crate::db::DB_POOL
        .get_connection()
        .map_err(|e| e.to_string())?;

    // Zde mapujeme chybu rusqlite::Error -> Box<dyn Error>
    let status: String = pooled_conn
        .execute(|conn| {
            conn.query_row(
                "SELECT status FROM copy_process WHERE id = ?1",
                [copy_process_id],
                |row| row.get(0),
            )
            .map_err(|err| Box::<dyn std::error::Error>::from(err))
        })
        // Následně mapujeme Box<dyn Error> na String pro návrat z funkce generate_report
        .map_err(|e| e.to_string())?;

    println!("Status z DB: {}", status);
    // Příklad: Získání informací o disku z lsblk (nutno upravit zařízení dle potřeby)
    let disk_info = crate::disk_utils::get_disk_info("/dev/sda")?;

    // Vytvoření struktury / Tera Context se stejnými poli jako v mock_data_report
    let mut context = Context::new();
    context.insert("software_hash", "75f1c14d734ea09147330fae210faa54");
    context.insert("build_date", "Jul 08, 2024 13:38:46 PDT");
    // Například sériové číslo z disk_info
    context.insert("serial_number", &disk_info.serial);
    context.insert("time_local", "12:17:32 (CEST +0200)");
    context.insert("date", "Jul 08, 2024");
    context.insert("mode", "DriveToFile");
    context.insert("method", "DDCapture");
    context.insert("hash_type", "SHA-1");
    context.insert("image_path", "/var/reports/.../Kingston480GB\\_SATA");
    context.insert("lba_count", &123456789u64);
    context.insert("sector_size", &512u64);
    context.insert("segment_size", "WholeDisk");
    context.insert("compression", "None");
    context.insert("hash_enabled", &true);
    context.insert("verify_hash", &false);
    // Např. HPA/DCO podle disk_info:
    context.insert("unlock_hpa", &disk_info.has_hpa);
    context.insert("unlock_dco", &disk_info.has_dco);
    context.insert("granularity", "SUCCESS");
    // Ukázka využití status z DB
    context.insert("result", &status);
    context.insert("duration", "00:47:20");
    context.insert("time_complete", "12:14:52");
    context.insert("hash_lba_count", &123456789u64);
    context.insert("hash_sector_size", &512u64);
    context.insert("hash_primary_type", "SHA-1");
    context.insert("hash_primary", "75f1c14d734ea09147330fae210faa54");
    context.insert("hash_secondary_type", "MD5");
    context.insert("hash_secondary", "abcdef1234567890abcdef1234567890");
    context.insert("case_file", "CaseFile\\_001");
    context.insert("case_id", "CASE-2024-001");
    context.insert("examiner", "John Doe");
    context.insert("notes", "This case involves encrypted drives.");
    context.insert("segment_uid", "SEG123456");
    context.insert("segment_path", "/segments/seg123456");
    context.insert("segment_fs", "NTFS");
    context.insert("segment_serial", "SEG-SERIAL-001");
    context.insert("segment_file", "segment\\_file.img");
    context.insert("segment_hash", "seg\\_hash\\_abc123");

    // Jen pro ukázku vytiskneme obsah celé struktury do konzole:
    println!("---[Výpis pro copy_process_id = {}]---", copy_process_id);
    println!("{:#?}", context);

    println!("Status: {}", status);

    Ok(())
}
