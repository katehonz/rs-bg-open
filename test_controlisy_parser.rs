use std::fs;

// Include the controlisy parsing module
include!("backend/src/services/controlisy.rs");

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Read the test XML file
    let xml_content = fs::read_to_string("test_controlisy_multiple.xml")?;

    println!("Testing XML parsing with {} bytes of content", xml_content.len());

    // Parse the XML
    match ControlisyService::parse_xml(&xml_content) {
        Ok(parsed_data) => {
            println!("✅ Successfully parsed XML");
            println!("📄 Number of contractors parsed: {}", parsed_data.contractors.len());
            println!("📄 Number of documents parsed: {}", parsed_data.documents.len());

            println!("\n=== CONTRACTORS ===");
            for (i, contractor) in parsed_data.contractors.iter().enumerate() {
                println!("Contractor {}: {} (EIK: {})",
                    i + 1,
                    contractor.contractor_name,
                    contractor.contractor_eik
                );
            }

            println!("\n=== DOCUMENTS ===");
            for (i, document) in parsed_data.documents.iter().enumerate() {
                println!("Document {}: {} - {} (Total: {:.2} BGN)",
                    i + 1,
                    document.document_number,
                    document.reason,
                    document.total_amount_bgn
                );
                println!("  Date: {}, VAT Month: {}",
                    document.document_date,
                    document.vat_month
                );
                println!("  Accounting entries: {}", document.accountings.len());

                // Show accounting details
                for (j, accounting) in document.accountings.iter().enumerate() {
                    println!("    Accounting {}: {:.2} BGN", j + 1, accounting.amount_bgn);
                    for (k, detail) in accounting.accounting_details.iter().enumerate() {
                        println!("      Detail {}: {} {} - {} {:.2}",
                            k + 1,
                            detail.direction,
                            detail.account_number,
                            detail.account_name,
                            accounting.amount_bgn
                        );
                    }
                }

                if let Some(vat_data) = &document.vat_data {
                    println!("  VAT Register: {}", vat_data.vat_register);
                    println!("  VAT entries: {}", vat_data.vat.len());
                }

                println!();
            }

            if parsed_data.documents.len() == 3 {
                println!("✅ All 3 documents were parsed successfully!");
            } else {
                println!("❌ Expected 3 documents but got {}", parsed_data.documents.len());
            }
        }
        Err(e) => {
            println!("❌ Failed to parse XML: {}", e);
            return Err(e.into());
        }
    }

    Ok(())
}