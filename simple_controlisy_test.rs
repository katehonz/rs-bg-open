use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Read the test XML file
    let xml_content = fs::read_to_string("test_controlisy_multiple.xml")?;

    println!("Testing XML content parsing...");
    println!("XML length: {} bytes", xml_content.len());

    // Count documents manually
    let document_count = xml_content.matches("<Document ").count();
    let contractor_count = xml_content.matches("<Contractor ").count();

    println!("Found {} <Document> tags", document_count);
    println!("Found {} <Contractor> tags", contractor_count);

    // Check specific document numbers
    if xml_content.contains("documentNumber=\"INV001\"") {
        println!("✅ Document INV001 found");
    } else {
        println!("❌ Document INV001 NOT found");
    }

    if xml_content.contains("documentNumber=\"INV002\"") {
        println!("✅ Document INV002 found");
    } else {
        println!("❌ Document INV002 NOT found");
    }

    if xml_content.contains("documentNumber=\"INV003\"") {
        println!("✅ Document INV003 found");
    } else {
        println!("❌ Document INV003 NOT found");
    }

    // Show XML structure
    let lines: Vec<&str> = xml_content.lines().collect();
    println!("\nXML structure:");
    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("<Document") ||
           trimmed.starts_with("</Document") ||
           trimmed.starts_with("<Contractor") ||
           trimmed.starts_with("<Documents") ||
           trimmed.starts_with("</Documents") {
            println!("Line {}: {}", i + 1, trimmed);
        }
    }

    Ok(())
}