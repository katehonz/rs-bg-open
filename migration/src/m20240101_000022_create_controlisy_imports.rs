use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create controlisy_imports table
        manager
            .create_table(
                Table::create()
                    .table(ControlisyImports::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ControlisyImports::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(ControlisyImports::CompanyId)
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ControlisyImports::ImportDate)
                            .timestamp()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ControlisyImports::FileName)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ControlisyImports::DocumentType)
                            .string()
                            .not_null(),
                    ) // 'purchase' or 'sale'
                    .col(ColumnDef::new(ControlisyImports::RawXml).text()) // Original XML content
                    .col(
                        ColumnDef::new(ControlisyImports::ParsedData)
                            .json()
                            .not_null(),
                    ) // Parsed JSON data
                    .col(
                        ColumnDef::new(ControlisyImports::Status)
                            .string()
                            .not_null()
                            .default("staged"),
                    ) // staged, reviewed, processing, completed, failed
                    .col(
                        ColumnDef::new(ControlisyImports::Processed)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(
                        ColumnDef::new(ControlisyImports::ReviewedAt)
                            .timestamp()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(ControlisyImports::ReviewedBy)
                            .integer()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(ControlisyImports::ProcessedAt)
                            .timestamp()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(ControlisyImports::ErrorMessage)
                            .text()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(ControlisyImports::ImportedDocuments)
                            .integer()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(ControlisyImports::ImportedContractors)
                            .integer()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(ControlisyImports::CreatedAt)
                            .timestamp()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ControlisyImports::UpdatedAt)
                            .timestamp()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_controlisy_imports_company")
                            .from(ControlisyImports::Table, ControlisyImports::CompanyId)
                            .to(Companies::Table, Companies::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Create index on company_id and status
        manager
            .create_index(
                Index::create()
                    .name("idx_controlisy_imports_company_status")
                    .table(ControlisyImports::Table)
                    .col(ControlisyImports::CompanyId)
                    .col(ControlisyImports::Status)
                    .to_owned(),
            )
            .await?;

        // Create controlisy_import_documents table for tracking individual documents
        manager
            .create_table(
                Table::create()
                    .table(ControlisyImportDocuments::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ControlisyImportDocuments::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(ControlisyImportDocuments::ImportId)
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ControlisyImportDocuments::CaDocId)
                            .string()
                            .not_null(),
                    ) // Controlisy document ID
                    .col(
                        ColumnDef::new(ControlisyImportDocuments::DocumentNumber)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ControlisyImportDocuments::DocumentDate)
                            .date()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ControlisyImportDocuments::ContractorId)
                            .integer()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(ControlisyImportDocuments::JournalEntryId)
                            .integer()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(ControlisyImportDocuments::VatEntryId)
                            .integer()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(ControlisyImportDocuments::TotalAmount)
                            .decimal_len(15, 2)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ControlisyImportDocuments::VatAmount)
                            .decimal_len(15, 2)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ControlisyImportDocuments::Status)
                            .string()
                            .not_null()
                            .default("pending"),
                    )
                    .col(
                        ColumnDef::new(ControlisyImportDocuments::ErrorMessage)
                            .text()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(ControlisyImportDocuments::CreatedAt)
                            .timestamp()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_import_documents_import")
                            .from(
                                ControlisyImportDocuments::Table,
                                ControlisyImportDocuments::ImportId,
                            )
                            .to(ControlisyImports::Table, ControlisyImports::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_import_documents_contractor")
                            .from(
                                ControlisyImportDocuments::Table,
                                ControlisyImportDocuments::ContractorId,
                            )
                            .to(Counterparts::Table, Counterparts::Id)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_import_documents_journal")
                            .from(
                                ControlisyImportDocuments::Table,
                                ControlisyImportDocuments::JournalEntryId,
                            )
                            .to(JournalEntries::Table, JournalEntries::Id)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .to_owned(),
            )
            .await?;

        // Create index on import_id and ca_doc_id
        manager
            .create_index(
                Index::create()
                    .name("idx_import_documents_import_doc")
                    .table(ControlisyImportDocuments::Table)
                    .col(ControlisyImportDocuments::ImportId)
                    .col(ControlisyImportDocuments::CaDocId)
                    .unique()
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(
                Table::drop()
                    .table(ControlisyImportDocuments::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(Table::drop().table(ControlisyImports::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(Iden)]
enum ControlisyImports {
    Table,
    Id,
    CompanyId,
    ImportDate,
    FileName,
    DocumentType,
    RawXml,
    ParsedData,
    Status,
    Processed,
    ReviewedAt,
    ReviewedBy,
    ProcessedAt,
    ErrorMessage,
    ImportedDocuments,
    ImportedContractors,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
enum ControlisyImportDocuments {
    Table,
    Id,
    ImportId,
    CaDocId,
    DocumentNumber,
    DocumentDate,
    ContractorId,
    JournalEntryId,
    VatEntryId,
    TotalAmount,
    VatAmount,
    Status,
    ErrorMessage,
    CreatedAt,
}

#[derive(Iden)]
enum Companies {
    Table,
    Id,
}

#[derive(Iden)]
enum Counterparts {
    Table,
    Id,
}

#[derive(Iden)]
enum JournalEntries {
    Table,
    Id,
}
