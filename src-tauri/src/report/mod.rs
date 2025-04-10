use printpdf::{
    BuiltinFont, Color, Line, LinePoint, Mm, Op, PdfDocument, PdfPage, PdfSaveOptions, Point, Pt, Rgb,
    TextItem,
};
use std::error::Error;
use std::fs;

// Přibližný koeficient převodu milimetrů na body (1 mm ≈ 2.8346457 pt)
const MM_TO_PT: f32 = 2.8346457;

/* ===========================
   1) Datové struktury reportu
   =========================== */

/// Sekce "Audit Log"
pub struct AuditLog {
    pub vendor: String,
    pub product: String,
    pub software_version: String,
    pub build_date: String,
    pub unit_serial_number: String,
    pub time_local: String,
    pub date: String,
}

/// Sekce "Operation Parameters"
pub struct OperationParameters {
    pub mode: String,
    pub method: String,
    pub captured_partition: u8,
    pub hash: String,
    pub image_path: String,
    pub lba_count: u64,
    pub source_logical_sector_size: u16,
    pub segment_size: String,
    pub compression_level: String,
    pub hash_enabled: bool,
    pub verify_hash: bool,
    pub unlock_hpa: bool,
    pub unlock_dco_acs3: bool,
    pub error_granularity: u8,
    pub result: String,
    pub duration: String,
    pub time_at_completion: String,
}

/// Sekce "Hash Information"
pub struct HashInformation {
    pub lba_count: u64,
    pub sector_size: u16,
    pub sha1: String,
    pub md5: String,
}

/// Sekce "Case Information"
pub struct CaseInformation {
    pub case_file_name: String,
    pub case_id: String,
    pub examiner: String,
    pub evidence_id: String,
    pub case_note: String,
}

/// Sekce "Segment Information"
pub struct SegmentInformation {
    pub uid: String,
    pub path: String,
    pub filesystem: String,
    pub serial: String,
    pub file_name: String,
    pub destination: String,
}

/// Řádek pro "Drive Information"
pub struct DriveInfoRow {
    pub bay: String,
    pub role: String,
    pub serial: String,
    pub model: String,
}

/// Řádek pro "Drive Capacities"
pub struct DriveCapacityRow {
    pub bay: String,
    pub model: String,
    pub serial: String,
    pub capacity: String,
}

/// Řádek pro "Drive ATA Security Information"
pub struct DriveAtaSecurityRow {
    pub bay: String,
    pub role: String,
    pub enabled: bool,
    pub locked: bool,
}

/// Řádek pro "Drive Encryption Information"
pub struct DriveEncryptionRow {
    pub bay: String,
    pub role: String,
    pub enabled: bool,
    pub locked: bool,
}

/// Řádek pro "Source Partition Information"
pub struct SourcePartitionRow {
    pub partition: String,
    pub file_system: String,
    pub start: u64,
    pub end: u64,
    pub size: u64,
    pub encryption: String,
    pub decrypted: bool,
    pub captured: bool,
}

/// Hlavní struktura obsahující všechna data reportu
pub struct PdfData {
    pub audit_log: AuditLog,
    pub operation_params: OperationParameters,
    pub hash_info: HashInformation,
    pub case_info: CaseInformation,
    pub segment_info: SegmentInformation,
    pub drive_info_rows: Vec<DriveInfoRow>,
    pub drive_capacity_rows: Vec<DriveCapacityRow>,
    pub drive_ata_security_rows: Vec<DriveAtaSecurityRow>,
    pub drive_encryption_rows: Vec<DriveEncryptionRow>,
    pub source_partition_rows: Vec<SourcePartitionRow>,
}

/* ======================================
   2) Pomocné funkce – kreslení textu & tabulek
   ====================================== */

/// Vykreslí horizontální linku (1 pt) v zadaných souřadnicích (v mm).
fn add_horizontal_line(ops: &mut Vec<Op>, x_start_mm: f32, x_end_mm: f32, y_mm: f32) {
    ops.push(Op::SaveGraphicsState);
    ops.push(Op::SetOutlineColor {
        col: Color::Rgb(Rgb { r: 0.0, g: 0.0, b: 0.0, icc_profile: None }),
    });
    ops.push(Op::SetOutlineThickness { pt: Pt(1.0) });
    ops.push(Op::DrawLine {
        line: Line {
            points: vec![
                LinePoint {
                    p: Point { x: Pt(x_start_mm * MM_TO_PT), y: Pt(y_mm * MM_TO_PT) },
                    bezier: false,
                },
                LinePoint {
                    p: Point { x: Pt(x_end_mm * MM_TO_PT), y: Pt(y_mm * MM_TO_PT) },
                    bezier: false,
                },
            ],
            is_closed: false,
        },
    });
    ops.push(Op::RestoreGraphicsState);
}

/// Vykreslí nadpis sekce (tučný text) a pod ním vodorovnou linku.
/// Aktualizuje ukazatel y (v mm) pro následný obsah.
fn add_section_heading(ops: &mut Vec<Op>, heading: &str, x_mm: f32, y_mm: &mut f32) {
    ops.push(Op::SaveGraphicsState);
    ops.push(Op::StartTextSection);
    ops.push(Op::SetTextCursor { pos: Point::new(Mm(x_mm), Mm(*y_mm)) });
    ops.push(Op::SetFontSizeBuiltinFont { size: Pt(14.0), font: BuiltinFont::HelveticaBold });
    ops.push(Op::WriteTextBuiltinFont { items: vec![TextItem::Text(heading.to_string())], font: BuiltinFont::HelveticaBold });
    ops.push(Op::EndTextSection);
    ops.push(Op::RestoreGraphicsState);

    *y_mm -= 2.0;
    add_horizontal_line(ops, x_mm, 190.0, *y_mm);
    *y_mm -= 8.0;
}

/// Vykreslí jeden řádek se dvojicí "label: value" (label tučně, value normálně).
fn add_label_value(ops: &mut Vec<Op>, label: &str, value: &str, x_mm: f32, y_mm: &mut f32, font_size_pt: f32) {
    ops.push(Op::SaveGraphicsState);
    ops.push(Op::StartTextSection);
    ops.push(Op::SetTextCursor { pos: Point::new(Mm(x_mm), Mm(*y_mm)) });
    ops.push(Op::SetFontSizeBuiltinFont { size: Pt(font_size_pt), font: BuiltinFont::HelveticaBold });
    ops.push(Op::WriteTextBuiltinFont { items: vec![TextItem::Text(label.to_string())], font: BuiltinFont::HelveticaBold });
    ops.push(Op::SetFontSizeBuiltinFont { size: Pt(font_size_pt), font: BuiltinFont::Helvetica });
    ops.push(Op::WriteTextBuiltinFont { items: vec![TextItem::Text(format!(" {}", value))], font: BuiltinFont::Helvetica });
    ops.push(Op::EndTextSection);
    ops.push(Op::RestoreGraphicsState);

    let line_height_mm = font_size_pt * 0.3528; // přibližně
    *y_mm -= line_height_mm;
}

/// Vykreslí obecnou tabulku, kde každý řádek je reprezentován vektorem stringů (jednotlivé buňky).
/// `col_widths` udávají šířky jednotlivých sloupců v mm, `cell_height_mm` výšku řádku.
fn add_generic_table(
    ops: &mut Vec<Op>,
    start_x_mm: f32,
    start_y_mm: &mut f32,
    col_widths: &[f32],
    rows: &[Vec<String>],
    cell_height_mm: f32,
) {
    let total_width_mm: f32 = col_widths.iter().sum();
    // Horní linka tabulky
    add_horizontal_line(ops, start_x_mm, start_x_mm + total_width_mm, *start_y_mm);

    for row in rows {
        let row_top = *start_y_mm;
        let row_bottom = row_top - cell_height_mm;
        let mut current_x = start_x_mm;

        for (col_index, cell_text) in row.iter().enumerate() {
            // Vykreslíme svislou linku mezi sloupci (kromě první buňky)
            if col_index > 0 {
                let x_line = current_x;
                ops.push(Op::SaveGraphicsState);
                ops.push(Op::SetOutlineColor {
                    col: Color::Rgb(Rgb { r: 0.0, g: 0.0, b: 0.0, icc_profile: None }),
                });
                ops.push(Op::SetOutlineThickness { pt: Pt(1.0) });
                ops.push(Op::DrawLine {
                    line: Line {
                        points: vec![
                            LinePoint {
                                p: Point { x: Pt(x_line * MM_TO_PT), y: Pt(row_top * MM_TO_PT) },
                                bezier: false,
                            },
                            LinePoint {
                                p: Point { x: Pt(x_line * MM_TO_PT), y: Pt(row_bottom * MM_TO_PT) },
                                bezier: false,
                            },
                        ],
                        is_closed: false,
                    },
                });
                ops.push(Op::RestoreGraphicsState);
            }

            // Vykreslíme text uvnitř buňky s malým odsazením
            ops.push(Op::SaveGraphicsState);
            ops.push(Op::StartTextSection);
            let text_x = current_x + 2.0;
            let text_y = row_top - cell_height_mm / 2.0 + 2.0;
            ops.push(Op::SetTextCursor { pos: Point::new(Mm(text_x), Mm(text_y)) });
            ops.push(Op::SetFontSizeBuiltinFont { size: Pt(11.0), font: BuiltinFont::Helvetica });
            ops.push(Op::WriteTextBuiltinFont { items: vec![TextItem::Text(cell_text.clone())], font: BuiltinFont::Helvetica });
            ops.push(Op::EndTextSection);
            ops.push(Op::RestoreGraphicsState);

            current_x += col_widths[col_index];
        }

        add_horizontal_line(ops, start_x_mm, start_x_mm + total_width_mm, row_bottom);
        *start_y_mm = row_bottom - 2.0; // mezera mezi řádky
    }
}

/* =======================================
   3) Funkce vytvářející PDF s rozdělením obsahu
   ======================================= */

pub fn create_pdf(pdf_data: PdfData, output_path: &str) -> Result<(), Box<dyn Error>> {
    // Budeme vytvářet dvě sady operací – jednu pro první stránku, druhou pro druhou stránku.
    let mut ops_page1: Vec<Op> = Vec::new();
    let mut ops_page2: Vec<Op> = Vec::new();

    // --- Obsah pro PRVNÍ STRÁNKU ---
    // Počáteční pozice pro první stránku (v mm)
    let mut current_y1 = 270.0;
    let start_x = 20.0;

    // Sekce Audit Log (vše vypíšeme jako label–value)
    add_section_heading(&mut ops_page1, "Audit Log", start_x, &mut current_y1);
    add_label_value(&mut ops_page1, "Vendor:", &pdf_data.audit_log.vendor, start_x, &mut current_y1, 12.0);
    add_label_value(&mut ops_page1, "Product:", &pdf_data.audit_log.product, start_x, &mut current_y1, 12.0);
    add_label_value(&mut ops_page1, "Software Version:", &pdf_data.audit_log.software_version, start_x, &mut current_y1, 12.0);
    add_label_value(&mut ops_page1, "Build Date:", &pdf_data.audit_log.build_date, start_x, &mut current_y1, 12.0);
    add_label_value(&mut ops_page1, "Unit Serial Number:", &pdf_data.audit_log.unit_serial_number, start_x, &mut current_y1, 12.0);
    add_label_value(&mut ops_page1, "Time (Local):", &pdf_data.audit_log.time_local, start_x, &mut current_y1, 12.0);
    add_label_value(&mut ops_page1, "Date:", &pdf_data.audit_log.date, start_x, &mut current_y1, 12.0);
    current_y1 -= 5.0;

    // Sekce Operation Parameters – zde do tabulky vložíme pouze vybrané parametry
    add_section_heading(&mut ops_page1, "Operation Parameters", start_x, &mut current_y1);
    // Vybrané parametry: Mode, Captured Partition, Hash, Image Path, LBA Count, Segment Size, Result.
    let captured_partition_str = pdf_data.operation_params.captured_partition.to_string();
    let lba_count_str = pdf_data.operation_params.lba_count.to_string();
    let selected_params = vec![
        vec!["Mode".to_string(), pdf_data.operation_params.mode.clone()],
        vec!["Captured Partition".to_string(), captured_partition_str],
        vec!["Hash".to_string(), pdf_data.operation_params.hash.clone()],
        vec!["Image Path".to_string(), pdf_data.operation_params.image_path.clone()],
        vec!["LBA Count".to_string(), lba_count_str],
        vec!["Segment Size".to_string(), pdf_data.operation_params.segment_size.clone()],
        vec!["Result".to_string(), pdf_data.operation_params.result.clone()],
    ];
    let op_params_col_widths = [45.0, 80.0];
    add_generic_table(&mut ops_page1, start_x, &mut current_y1, &op_params_col_widths, &selected_params, 10.0);

    // --- Obsah pro DRUHOU STRÁNKU ---
    // Počáteční pozice pro druhou stránku (v mm)
    let mut current_y2 = 270.0;

    // Na druhé stránce zobrazíme zbytek sekcí
    // Sekce Hash Information
    add_section_heading(&mut ops_page2, "Hash Information", start_x, &mut current_y2);
    let hash_rows = vec![
        vec!["LBA Count".to_string(), pdf_data.hash_info.lba_count.to_string()],
        vec!["Sector Size".to_string(), pdf_data.hash_info.sector_size.to_string()],
        vec!["SHA-1".to_string(), pdf_data.hash_info.sha1.clone()],
        vec!["MD5".to_string(), pdf_data.hash_info.md5.clone()],
    ];
    add_generic_table(&mut ops_page2, start_x, &mut current_y2, &op_params_col_widths, &hash_rows, 10.0);

    // Sekce Case Information
    add_section_heading(&mut ops_page2, "Case Information", start_x, &mut current_y2);
    let case_rows = vec![
        vec!["Case/File Name".to_string(), pdf_data.case_info.case_file_name.clone()],
        vec!["Case ID".to_string(), pdf_data.case_info.case_id.clone()],
        vec!["Examiner".to_string(), pdf_data.case_info.examiner.clone()],
        vec!["Evidence ID".to_string(), pdf_data.case_info.evidence_id.clone()],
        vec!["Case Note".to_string(), pdf_data.case_info.case_note.clone()],
    ];
    add_generic_table(&mut ops_page2, start_x, &mut current_y2, &op_params_col_widths, &case_rows, 10.0);

    // Sekce Segment Information
    add_section_heading(&mut ops_page2, "Segment Information", start_x, &mut current_y2);
    let seg_rows = vec![
        vec!["UID".to_string(), pdf_data.segment_info.uid.clone()],
        vec!["Path".to_string(), pdf_data.segment_info.path.clone()],
        vec!["Filesystem".to_string(), pdf_data.segment_info.filesystem.clone()],
        vec!["Serial".to_string(), pdf_data.segment_info.serial.clone()],
        vec!["File Name".to_string(), pdf_data.segment_info.file_name.clone()],
        vec!["Destination".to_string(), pdf_data.segment_info.destination.clone()],
    ];
    add_generic_table(&mut ops_page2, start_x, &mut current_y2, &op_params_col_widths, &seg_rows, 10.0);

    // Sekce Drive Information
    add_section_heading(&mut ops_page2, "Drive Information", start_x, &mut current_y2);
    let drive_info_header = vec!["Bay".to_string(), "Role".to_string(), "Serial".to_string(), "Model".to_string()];
    let mut drive_info_rows_str: Vec<Vec<String>> = Vec::new();
    drive_info_rows_str.push(drive_info_header);
    for row in &pdf_data.drive_info_rows {
        drive_info_rows_str.push(vec![
            row.bay.clone(),
            row.role.clone(),
            row.serial.clone(),
            row.model.clone(),
        ]);
    }
    let drive_info_col_widths = [25.0, 25.0, 60.0, 40.0];
    add_generic_table(&mut ops_page2, start_x, &mut current_y2, &drive_info_col_widths, &drive_info_rows_str, 10.0);

    // Můžete dále přidávat další sekce (Drive Capacities, ATA Security, Encryption, Source Partition, atd.)
    // Podobně jako výše – v závislosti na tom, kolik informací chcete na druhou stránku.
    // Pro tento příklad přidáme ještě Drive Capacities:

    add_section_heading(&mut ops_page2, "Drive Capacities", start_x, &mut current_y2);
    let mut drive_cap_rows: Vec<Vec<String>> = Vec::new();
    drive_cap_rows.push(vec![
        "Bay".to_string(),
        "Model".to_string(),
        "Serial".to_string(),
        "Capacity".to_string(),
    ]);
    for row in &pdf_data.drive_capacity_rows {
        drive_cap_rows.push(vec![
            row.bay.clone(),
            row.model.clone(),
            row.serial.clone(),
            row.capacity.clone(),
        ]);
    }
    let drive_cap_col_widths = [25.0, 40.0, 60.0, 25.0];
    add_generic_table(&mut ops_page2, start_x, &mut current_y2, &drive_cap_col_widths, &drive_cap_rows, 10.0);

    // --- Vytvoření PDF dokumentu se dvěma stránkami ---
    let page1 = PdfPage::new(Mm(210.0), Mm(297.0), ops_page1);
    let page2 = PdfPage::new(Mm(210.0), Mm(297.0), ops_page2);
    let mut doc = PdfDocument::new("Custom Two-Page Report");
    let pdf_bytes = doc.with_pages(vec![page1, page2]).save(&PdfSaveOptions::default(), &mut Vec::new());
    fs::write(output_path, pdf_bytes)?;

    Ok(())
}

/* =================================================
   4) Veřejná funkce run_sample() – naplní data a vygeneruje report
   ================================================= */

pub fn run_sample() -> Result<(), Box<dyn Error>> {
    // Ukázková data – upravte dle potřeby

    let audit_log = AuditLog {
        vendor: "Logicube".into(),
        product: "Falcon-Neo".into(),
        software_version: "3.3".into(),
        build_date: "Jul 08, 2021 13:38:46 PDT".into(),
        unit_serial_number: "171945".into(),
        time_local: "11:27:32 (CEST +0200)".into(),
        date: "Jul 08, 2024".into(),
    };

    let operation_params = OperationParameters {
        mode: "DriveToFile".into(),
        method: "DDCapture".into(),
        captured_partition: 0,
        hash: "SHA-1+MD5".into(),
        image_path: "Jvar/repo/sata-d3/KRPS-172758-TC-2024-011281/Kingston480GB_SATA".into(),
        lba_count: 937_703_088,
        source_logical_sector_size: 512,
        segment_size: "WholeDisk".into(),
        compression_level: "None".into(),
        hash_enabled: true,
        verify_hash: false,
        unlock_hpa: true,
        unlock_dco_acs3: true,
        error_granularity: 1,
        result: "SUCCESS".into(),
        duration: "00:47:20".into(),
        time_at_completion: "12:14:52".into(),
    };

    let hash_info = HashInformation {
        lba_count: 937_703_088,
        sector_size: 512,
        sha1: "688553de239242293567bh478797d288c7eass".into(),
        md5: "f32¢5d¢b0177f089fd2fb858b2c2252d".into(),
    };

    let case_info = CaseInformation {
        case_file_name: "Kingston480GB_SATA".into(),
        case_id: "KRPS-172758-TC-2024-011281".into(),
        examiner: "M".into(),
        evidence_id: "".into(),
        case_note: "".into(),
    };

    let segment_info = SegmentInformation {
        uid: "SATA_D3/0D88D21325B5D3A2".into(),
        path: "/var/repo/sata-d3/KRPS-172758-TC-2024-011281/Kingston480GB_SATA".into(),
        filesystem: "ntfs".into(),
        serial: "WD-WCC131932709".into(),
        file_name: "Kingston480GB_SATA.001".into(),
        destination: "".into(),
    };

    let drive_info_rows = vec![DriveInfoRow {
        bay: "SAS_S1".into(),
        role: "S".into(),
        serial: "50026B778371D594".into(),
        model: "KINGSTON_SA400S37480G".into(),
    }];

    let drive_capacity_rows = vec![
        DriveCapacityRow {
            bay: "SAS_S1".into(),
            model: "KINGSTON_SA400S37480G".into(),
            serial: "50026B778371D594".into(),
            capacity: "480.1 GB".into(),
        },
        DriveCapacityRow {
            bay: "SATA_D3".into(),
            model: "WDC_WD3000F9YZ-09N20L0".into(),
            serial: "WD-WCC131932709".into(),
            capacity: "3000.6 GB".into(),
        },
    ];

    let drive_ata_security_rows = vec![
        DriveAtaSecurityRow {
            bay: "SAS_S1".into(),
            role: "S".into(),
            enabled: false,
            locked: false,
        },
        DriveAtaSecurityRow {
            bay: "SATA_D3".into(),
            role: "D".into(),
            enabled: false,
            locked: false,
        },
    ];

    let drive_encryption_rows = vec![
        DriveEncryptionRow {
            bay: "SAS_S1".into(),
            role: "S".into(),
            enabled: false,
            locked: false,
        },
        DriveEncryptionRow {
            bay: "SATA_D3".into(),
            role: "D".into(),
            enabled: false,
            locked: false,
        },
    ];

    let source_partition_rows = vec![
        SourcePartitionRow {
            partition: "1".into(),
            file_system: "FAT32".into(),
            start: 1_048_576,
            end: 105_905_664,
            size: 104_857_600,
            encryption: "None".into(),
            decrypted: false,
            captured: true,
        },
        SourcePartitionRow {
            partition: "2".into(),
            file_system: "NTFS".into(),
            start: 105_906_176,
            end: 122_682_880,
            size: 16_777_216,
            encryption: "None".into(),
            decrypted: false,
            captured: true,
        },
    ];

    let pdf_data = PdfData {
        audit_log,
        operation_params,
        hash_info,
        case_info,
        segment_info,
        drive_info_rows,
        drive_capacity_rows,
        drive_ata_security_rows,
        drive_encryption_rows,
        source_partition_rows,
    };

    create_pdf(pdf_data, "/home/cratec/full_report.pdf")
}
