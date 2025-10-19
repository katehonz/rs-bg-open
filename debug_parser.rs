use std::fs;

fn debug_parse_xml(xml_content: &str) {
    println!("=== DEBUG XML PARSING ===");

    // Simple manual parsing
    let mut contractors_count = 0;
    let mut documents_count = 0;
    let mut in_documents = false;
    let mut in_contractors = false;
    let mut current_document_number = String::new();

    let lines: Vec<&str> = xml_content.lines().collect();

    for (line_num, line) in lines.iter().enumerate() {
        let trimmed = line.trim();

        if trimmed.starts_with("<Contractors>") {
            in_contractors = true;
            println!("Line {}: Started Contractors section", line_num + 1);
        } else if trimmed.starts_with("</Contractors>") {
            in_contractors = false;
            println!("Line {}: Ended Contractors section", line_num + 1);
        } else if trimmed.starts_with("<Documents>") {
            in_documents = true;
            println!("Line {}: Started Documents section", line_num + 1);
        } else if trimmed.starts_with("</Documents>") {
            in_documents = false;
            println!("Line {}: Ended Documents section", line_num + 1);
        } else if trimmed.starts_with("<Contractor ") && in_contractors {
            contractors_count += 1;
            if let Some(start) = trimmed.find("contractorName=\"") {
                let start = start + 16;
                if let Some(end) = trimmed[start..].find("\"") {
                    let name = &trimmed[start..start + end];
                    println!("Line {}: Found Contractor {}: {}", line_num + 1, contractors_count, name);
                }
            }
        } else if trimmed.starts_with("<Document") && in_documents {
            documents_count += 1;
            println!("Line {}: Started Document {} parsing", line_num + 1, documents_count);
            // Document attributes may be on multiple lines, so we'll track the document number later
        }

        // Check for document number on any line while in documents section
        if in_documents && trimmed.contains("documentNumber=") {
            println!("Line {} content: '{}'", line_num + 1, trimmed);
            if let Some(start) = trimmed.find("documentNumber=\"") {
                let start_pos = start + "documentNumber=\"".len();
                println!("Looking for closing quote starting from position {}", start_pos);
                println!("Remaining string: '{}'", &trimmed[start_pos..]);
                if let Some(end) = trimmed[start_pos..].find("\"") {
                    let number = &trimmed[start_pos..start_pos + end];
                    println!("Line {}: Found Document Number: '{}'", line_num + 1, number);
                    current_document_number = number.to_string();
                } else {
                    println!("Line {}: Could not find closing quote", line_num + 1);
                }
            } else {
                println!("Line {}: Could not find documentNumber= pattern", line_num + 1);
            }
        } else if trimmed.starts_with("</Document>") && in_documents {
            println!("Line {}: Ended Document {} ({})", line_num + 1, documents_count, current_document_number);
        }
    }

    println!("\n=== SUMMARY ===");
    println!("Total contractors found: {}", contractors_count);
    println!("Total documents found: {}", documents_count);

    // Also do a simple count
    let simple_contractors = xml_content.matches("<Contractor ").count();
    let simple_documents_start = xml_content.matches("<Document").count();
    let simple_documents_end = xml_content.matches("</Document>").count();

    println!("\n=== SIMPLE COUNTS ===");
    println!("Simple contractor count: {}", simple_contractors);
    println!("Simple document start count: {}", simple_documents_start);
    println!("Simple document end count: {}", simple_documents_end);
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let xml_content = fs::read_to_string("test_controlisy_multiple.xml")?;

    debug_parse_xml(&xml_content);

    Ok(())
}