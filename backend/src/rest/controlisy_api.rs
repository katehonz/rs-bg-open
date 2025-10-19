use actix_web::{get, post, web, HttpResponse, Result as ActixResult};
use base64::{engine::general_purpose, Engine as _};
use chrono;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, Set,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::entities::controlisy_imports;

use crate::services::controlisy::ControlisyService;

#[derive(Deserialize)]
pub struct ImportFileRequest {
    pub company_id: i32,
    pub file_name: String,
    pub xml_content: String,
}

#[derive(Deserialize)]
pub struct UpdateImportRequest {
    pub file_name: Option<String>,
    pub status: Option<String>,
    pub parsed_data: Option<serde_json::Value>,
}

#[derive(Serialize)]
pub struct ImportFileResponse {
    pub success: bool,
    pub import_id: i32,
    pub message: String,
    pub document_type: String,
    pub documents_count: i32,
    pub contractors_count: i32,
}

#[derive(Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

#[derive(Serialize)]
pub struct ImportListItem {
    pub id: i32,
    pub file_name: String,
    pub document_type: String,
    pub status: String,
    pub import_date: String,
    pub imported_documents: i32,
    pub imported_contractors: i32,
}

/// REST API endpoint for importing Controlisy XML files
pub async fn import_file(
    req: web::Json<ImportFileRequest>,
    db: web::Data<Arc<DatabaseConnection>>,
) -> ActixResult<HttpResponse> {
    let db = db.as_ref();

    // Auto-determine document type from file name
    let document_type = ControlisyService::determine_document_type(&req.file_name);
    if document_type == "unknown" {
        return Ok(HttpResponse::BadRequest().json(ErrorResponse {
            error: "Could not determine document type from file name. File name should contain 'pokupki', 'prodaj' or similar.".to_string(),
        }));
    }

    // Decode base64 XML content first
    let xml_content_decoded = match general_purpose::STANDARD.decode(&req.xml_content) {
        Ok(decoded) => match String::from_utf8(decoded) {
            Ok(xml_str) => xml_str,
            Err(e) => {
                return Ok(HttpResponse::BadRequest().json(ErrorResponse {
                    error: format!("Failed to decode XML as UTF-8: {}", e),
                }));
            }
        },
        Err(e) => {
            return Ok(HttpResponse::BadRequest().json(ErrorResponse {
                error: format!("Failed to decode base64 content: {}", e),
            }));
        }
    };

    // Parse XML first to validate
    let parsed_data = match ControlisyService::parse_xml(&xml_content_decoded) {
        Ok(data) => data,
        Err(e) => {
            return Ok(HttpResponse::BadRequest().json(ErrorResponse {
                error: format!("Failed to parse XML: {}", e),
            }));
        }
    };

    // Import to database
    let import_id = match ControlisyService::import_to_database(
        db,
        req.company_id,
        &req.file_name,
        &document_type,
        &xml_content_decoded,
    )
    .await
    {
        Ok(id) => id,
        Err(e) => {
            return Ok(HttpResponse::InternalServerError().json(ErrorResponse {
                error: format!("Failed to import: {}", e),
            }));
        }
    };

    Ok(HttpResponse::Ok().json(ImportFileResponse {
        success: true,
        import_id,
        message: format!(
            "Successfully imported {} documents and {} contractors",
            parsed_data.documents.len(),
            parsed_data.contractors.len()
        ),
        document_type,
        documents_count: parsed_data.documents.len() as i32,
        contractors_count: parsed_data.contractors.len() as i32,
    }))
}

/// REST API endpoint for listing imports
pub async fn list_imports(
    company_id: web::Path<i32>,
    db: web::Data<Arc<DatabaseConnection>>,
) -> ActixResult<HttpResponse> {
    let db = db.as_ref().as_ref();

    match controlisy_imports::Entity::find()
        .filter(controlisy_imports::Column::CompanyId.eq(*company_id))
        .order_by_desc(controlisy_imports::Column::ImportDate)
        .all(db)
        .await
    {
        Ok(imports) => {
            let import_list: Vec<ImportListItem> = imports
                .into_iter()
                .map(|import| ImportListItem {
                    id: import.id,
                    file_name: import.file_name,
                    document_type: import.document_type,
                    status: import.status,
                    import_date: import.import_date.format("%Y-%m-%d %H:%M:%S").to_string(),
                    imported_documents: import.imported_documents.unwrap_or(0),
                    imported_contractors: import.imported_contractors.unwrap_or(0),
                })
                .collect();

            Ok(HttpResponse::Ok().json(import_list))
        }
        Err(e) => {
            eprintln!("Error fetching imports: {}", e);
            Ok(HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Failed to fetch import list".to_string(),
            }))
        }
    }
}

/// REST API endpoint for parsing XML without importing
pub async fn parse_file(req: web::Json<serde_json::Value>) -> ActixResult<HttpResponse> {
    let xml_content = match req.get("xml_content").and_then(|v| v.as_str()) {
        Some(content) => content,
        None => {
            return Ok(HttpResponse::BadRequest().json(ErrorResponse {
                error: "Missing xml_content field".to_string(),
            }));
        }
    };

    // Decode base64 XML content
    let xml_content_decoded = match general_purpose::STANDARD.decode(xml_content) {
        Ok(decoded) => match String::from_utf8(decoded) {
            Ok(xml_str) => xml_str,
            Err(e) => {
                return Ok(HttpResponse::BadRequest().json(ErrorResponse {
                    error: format!("Failed to decode XML as UTF-8: {}", e),
                }));
            }
        },
        Err(e) => {
            return Ok(HttpResponse::BadRequest().json(ErrorResponse {
                error: format!("Failed to decode base64 content: {}", e),
            }));
        }
    };

    let file_name = req.get("file_name").and_then(|v| v.as_str()).unwrap_or("");
    let document_type = ControlisyService::determine_document_type(file_name);

    match ControlisyService::parse_xml(&xml_content_decoded) {
        Ok(parsed_data) => Ok(HttpResponse::Ok().json(serde_json::json!({
            "success": true,
            "document_type": document_type,
            "documents_count": parsed_data.documents.len(),
            "contractors_count": parsed_data.contractors.len(),
            "data": parsed_data
        }))),
        Err(e) => Ok(HttpResponse::BadRequest().json(ErrorResponse {
            error: format!("Failed to parse XML: {}", e),
        })),
    }
}

/// REST API endpoint for processing staged import
pub async fn process_import(
    import_id: web::Path<i32>,
    db: web::Data<Arc<DatabaseConnection>>,
) -> ActixResult<HttpResponse> {
    let db = db.as_ref();

    match ControlisyService::process_import(db, *import_id).await {
        Ok(_) => Ok(HttpResponse::Ok().json(serde_json::json!({
            "success": true,
            "message": "Import processed successfully"
        }))),
        Err(e) => Ok(HttpResponse::InternalServerError().json(ErrorResponse {
            error: format!("Failed to process import: {}", e),
        })),
    }
}

/// REST API endpoint for updating staged data
pub async fn update_staged_data(
    import_id: web::Path<i32>,
    req: web::Json<serde_json::Value>,
    db: web::Data<Arc<DatabaseConnection>>,
) -> ActixResult<HttpResponse> {
    let db = db.as_ref();

    let updated_data = match req.get("data").and_then(|v| v.as_str()) {
        Some(data) => data,
        None => {
            return Ok(HttpResponse::BadRequest().json(ErrorResponse {
                error: "Missing data field".to_string(),
            }));
        }
    };

    match ControlisyService::update_staged_data(db, *import_id, updated_data).await {
        Ok(_) => Ok(HttpResponse::Ok().json(serde_json::json!({
            "success": true,
            "message": "Staged data updated successfully"
        }))),
        Err(e) => Ok(HttpResponse::InternalServerError().json(ErrorResponse {
            error: format!("Failed to update staged data: {}", e),
        })),
    }
}

/// REST API endpoint for marking import as reviewed
pub async fn mark_reviewed(
    import_id: web::Path<i32>,
    req: web::Json<serde_json::Value>,
    db: web::Data<Arc<DatabaseConnection>>,
) -> ActixResult<HttpResponse> {
    let db = db.as_ref();

    let user_id = match req.get("user_id").and_then(|v| v.as_i64()) {
        Some(id) => id as i32,
        None => {
            return Ok(HttpResponse::BadRequest().json(ErrorResponse {
                error: "Missing user_id field".to_string(),
            }));
        }
    };

    match ControlisyService::mark_as_reviewed(db, *import_id, user_id).await {
        Ok(_) => Ok(HttpResponse::Ok().json(serde_json::json!({
            "success": true,
            "message": "Import marked as reviewed successfully"
        }))),
        Err(e) => Ok(HttpResponse::InternalServerError().json(ErrorResponse {
            error: format!("Failed to mark as reviewed: {}", e),
        })),
    }
}

/// REST API endpoint for deleting import
pub async fn delete_import(
    import_id: web::Path<i32>,
    db: web::Data<Arc<DatabaseConnection>>,
) -> ActixResult<HttpResponse> {
    let db = db.as_ref().as_ref();

    match controlisy_imports::Entity::delete_by_id(*import_id)
        .exec(db)
        .await
    {
        Ok(delete_result) => {
            if delete_result.rows_affected > 0 {
                Ok(HttpResponse::Ok().json(serde_json::json!({
                    "success": true,
                    "message": "Import deleted successfully"
                })))
            } else {
                Ok(HttpResponse::NotFound().json(ErrorResponse {
                    error: "Import not found".to_string(),
                }))
            }
        }
        Err(e) => {
            eprintln!("Error deleting import: {}", e);
            Ok(HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Failed to delete import".to_string(),
            }))
        }
    }
}

/// REST API endpoint for getting single import by ID
pub async fn get_import(
    import_id: web::Path<i32>,
    db: web::Data<Arc<DatabaseConnection>>,
) -> ActixResult<HttpResponse> {
    let db = db.as_ref().as_ref();

    match controlisy_imports::Entity::find_by_id(*import_id)
        .one(db)
        .await
    {
        Ok(Some(import)) => Ok(HttpResponse::Ok().json(serde_json::json!({
            "id": import.id,
            "file_name": import.file_name,
            "document_type": import.document_type,
            "status": import.status,
            "import_date": import.import_date.format("%Y-%m-%d %H:%M:%S").to_string(),
            "imported_documents": import.imported_documents.unwrap_or(0),
            "imported_contractors": import.imported_contractors.unwrap_or(0),
            "raw_xml": import.raw_xml,
            "parsed_data": import.parsed_data
        }))),
        Ok(None) => Ok(HttpResponse::NotFound().json(ErrorResponse {
            error: "Import not found".to_string(),
        })),
        Err(e) => {
            eprintln!("Error fetching import: {}", e);
            Ok(HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Failed to fetch import".to_string(),
            }))
        }
    }
}

/// REST API endpoint for updating import
pub async fn update_import(
    import_id: web::Path<i32>,
    req: web::Json<UpdateImportRequest>,
    db: web::Data<Arc<DatabaseConnection>>,
) -> ActixResult<HttpResponse> {
    let db = db.as_ref().as_ref();

    // First, fetch the existing import
    let existing_import = match controlisy_imports::Entity::find_by_id(*import_id)
        .one(db)
        .await
    {
        Ok(Some(import)) => import,
        Ok(None) => {
            return Ok(HttpResponse::NotFound().json(ErrorResponse {
                error: "Import not found".to_string(),
            }));
        }
        Err(e) => {
            eprintln!("Error fetching import: {}", e);
            return Ok(HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Failed to fetch import".to_string(),
            }));
        }
    };

    // Create active model for update
    let mut active_model = controlisy_imports::ActiveModel {
        id: Set(existing_import.id),
        ..Default::default()
    };

    // Update fields if provided
    if let Some(file_name) = &req.file_name {
        active_model.file_name = Set(file_name.clone());
    }

    if let Some(status) = &req.status {
        active_model.status = Set(status.clone());
    }

    if let Some(parsed_data) = &req.parsed_data {
        active_model.parsed_data = Set(parsed_data.clone());
    }

    // Always update the updated_at timestamp
    active_model.updated_at = Set(chrono::Local::now().naive_local());

    // Save the updated import
    match active_model.update(db).await {
        Ok(updated_import) => Ok(HttpResponse::Ok().json(serde_json::json!({
            "success": true,
            "message": "Import updated successfully",
            "import": {
                "id": updated_import.id,
                "file_name": updated_import.file_name,
                "document_type": updated_import.document_type,
                "status": updated_import.status,
                "import_date": updated_import.import_date.format("%Y-%m-%d %H:%M:%S").to_string(),
                "imported_documents": updated_import.imported_documents.unwrap_or(0),
                "imported_contractors": updated_import.imported_contractors.unwrap_or(0)
            }
        }))),
        Err(e) => {
            eprintln!("Error updating import: {}", e);
            Ok(HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Failed to update import".to_string(),
            }))
        }
    }
}
