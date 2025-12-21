// Test script for SAF-T v1.0.1 implementation
use backend::entities::saft::{SafTExportRequest, SafTFileType};
use backend::services::saft_service_v2::SafTServiceV2;
use sea_orm::{Database, DatabaseConnection};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Connect to database (using empty string for test)
    let db: DatabaseConnection = Database::connect("sqlite::memory:").await?;
    
    // Create SAF-T service
    let saft_service = SafTServiceV2::new(db);
    
    // Create test export request
    let request = SafTExportRequest {
        company_id: 1,
        period_start: 11,
        period_start_year: 2024,
        period_end: 12,
        period_end_year: 2024,
        file_type: SafTFileType::Monthly,
        tax_accounting_basis: "A".to_string(),
    };
    
    // Test structure creation (won't work without real data, but will test compilation)
    match saft_service.generate_saft(request).await {
        Ok(xml) => {
            println!("SAF-T XML generated successfully");
            println!("Length: {} bytes", xml.len());
            // Print first 500 characters
            if xml.len() > 500 {
                println!("First 500 chars:\n{}", &xml[0..500]);
            } else {
                println!("Content:\n{}", xml);
            }
        }
        Err(e) => {
            println!("Error generating SAF-T: {}", e);
        }
    }
    
    Ok(())
}