use std::fs;

fn parse_xml_simple(xml_content: &str) -> Result<(usize, usize), String> {
    let mut contractors: Vec<String> = Vec::new();
    let mut documents: Vec<String> = Vec::new();

    let lines: Vec<&str> = xml_content.lines().collect();
    let mut in_documents = false;
    let mut document_count = 0;
    let mut contractor_count = 0;

    for line in lines {
        let trimmed = line.trim();

        if trimmed.starts_with("<Contractors>") {
            println!("Found Contractors section");
        } else if trimmed.starts_with("<Documents>") {
            println!("Found Documents section");
            in_documents = true;
        } else if trimmed.starts_with("<Contractor ") {
            contractor_count += 1;
            // Extract contractor name
            if let Some(start) = trimmed.find("contractorName=\"") {
                let start = start + 16;
                if let Some(end) = trimmed[start..].find("\"") {
                    let name = &trimmed[start..start + end];
                    println!("  Contractor {}: {}", contractor_count, name);
                }
            }
        } else if trimmed.starts_with("<Document ") && in_documents {
            document_count += 1;
            // Extract document number
            if let Some(start) = trimmed.find("documentNumber=\"") {
                let start = start + 15;
                if let Some(end) = trimmed[start..].find("\"") {
                    let number = &trimmed[start..start + end];
                    println!("  Document {}: {}", document_count, number);
                }
            }
        } else if trimmed.starts_with("</Documents>") {
            in_documents = false;
            println!("End of Documents section");
        }
    }

    println!("Total: {} contractors, {} documents", contractor_count, document_count);
    Ok((contractor_count, document_count))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Read the test XML file
    let xml_content = fs::read_to_string("test_controlisy_multiple.xml")?;

    println!("Testing simple XML parsing...");

    match parse_xml_simple(&xml_content) {
        Ok((contractors, documents)) => {
            println!("\n✅ Parse completed:");
            println!("   Contractors: {}", contractors);
            println!("   Documents: {}", documents);

            if documents == 3 {
                println!("✅ Expected 3 documents found!");
            } else {
                println!("❌ Expected 3 documents but found {}", documents);
            }
        }
        Err(e) => {
            println!("❌ Parse failed: {}", e);
        }
    }

    Ok(())
}