use std::fs;

fn debug_parse_real_xml(xml_content: &str) {
    println!("=== REAL CONTROLISY XML PARSING ===");

    let mut contractors_count = 0;
    let mut documents_count = 0;
    let mut in_documents = false;
    let mut in_contractors = false;

    let lines: Vec<&str> = xml_content.lines().collect();

    for (line_num, line) in lines.iter().enumerate() {
        let trimmed = line.trim();

        if trimmed.starts_with("<Contractors>") {
            in_contractors = true;
            println!("Line {}: Started Contractors section", line_num + 1);
        } else if trimmed.starts_with("</Contractors>") {
            in_contractors = false;
            println!("Line {}: Ended Contractors section - Total: {}", line_num + 1, contractors_count);
        } else if trimmed.starts_with("<Documents>") {
            in_documents = true;
            println!("Line {}: Started Documents section", line_num + 1);
        } else if trimmed.starts_with("</Documents>") {
            in_documents = false;
            println!("Line {}: Ended Documents section - Total: {}", line_num + 1, documents_count);
        } else if trimmed.starts_with("<Contractor ") && in_contractors {
            contractors_count += 1;
            if let Some(start) = trimmed.find("contractorName=\"") {
                let start = start + "contractorName=\"".len();
                if let Some(end) = trimmed[start..].find("\"") {
                    let name = &trimmed[start..start + end];
                    println!("  Contractor {}: {}", contractors_count, name);
                }
            }
        } else if trimmed.starts_with("<Document ") && in_documents {
            documents_count += 1;
            // Extract document number from the line or subsequent lines
            let mut doc_number = String::new();

            // Look for documentNumber on this line or the next few lines
            for check_line in &lines[line_num..std::cmp::min(line_num + 10, lines.len())] {
                if check_line.trim().contains("documentNumber=") {
                    if let Some(start) = check_line.find("documentNumber=\"") {
                        let start = start + "documentNumber=\"".len();
                        if let Some(end) = check_line[start..].find("\"") {
                            doc_number = check_line[start..start + end].to_string();
                            break;
                        }
                    }
                }
            }

            println!("  Document {}: {}", documents_count, doc_number);

            // Print every 5th document progress
            if documents_count % 5 == 0 {
                println!("    ... processed {} documents so far", documents_count);
            }
        }
    }

    println!("\n=== SUMMARY ===");
    println!("Total contractors found: {}", contractors_count);
    println!("Total documents found: {}", documents_count);

    // Verification counts
    let simple_contractors = xml_content.matches("<Contractor ").count();
    let simple_documents = xml_content.matches("<Document ").count();

    println!("\n=== VERIFICATION ===");
    println!("Simple contractor count: {}", simple_contractors);
    println!("Simple document count: {}", simple_documents);

    if contractors_count == simple_contractors {
        println!("✅ Contractor parsing is correct");
    } else {
        println!("❌ Contractor parsing mismatch");
    }

    if documents_count == simple_documents {
        println!("✅ Document parsing is correct");
    } else {
        println!("❌ Document parsing mismatch");
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let xml_content = fs::read_to_string("z-cont/contr/prodajbi-07.2025_02-Продажби.xml")?;

    println!("Reading real Controlisy XML file...");
    println!("File size: {} bytes", xml_content.len());

    debug_parse_real_xml(&xml_content);

    Ok(())
}