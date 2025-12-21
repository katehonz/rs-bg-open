# S3/Object Storage Implementation - Complete Analysis Index

This directory contains comprehensive documentation of the S3/object storage implementation in the rs-ac-bg (Rust Accounting Backend) application.

## Documentation Files

### 1. S3_IMPLEMENTATION_ANALYSIS.md (21 KB)
**Comprehensive technical analysis covering:**
- Configuration and credentials setup
- Client initialization and configuration flow
- Configuration persistence and updates
- File upload logic (database backups)
- Backup retrieval and download logic
- Error handling and logging
- Compatibility and provider support
- GraphQL API integration
- Complete data flow diagrams
- Security considerations
- Summary tables
- Potential enhancements

**Best for:** Understanding the complete architecture and implementation details

### 2. S3_CODE_SNIPPETS.md (19 KB)
**Annotated code extracts featuring:**
- Key file locations (with full paths)
- Configuration structures
- Maintenance service structures
- S3 client building code
- Upload implementation
- Backup creation (integrated flow)
- Listing remote backups
- Download for restore
- Settings persistence
- GraphQL resolvers
- Dependencies and imports
- Configuration JSON examples

**Best for:** Learning specific implementations and copy-paste reference

### 3. S3_QUICK_REFERENCE.md (9.9 KB)
**Quick lookup guide including:**
- File paths table
- Key concepts summary
- Configuration setup steps
- S3 API usage patterns
- Data flow diagrams
- Error handling checklist
- GraphQL operations (with examples)
- Testing configurations (AWS, Hetzner, MinIO)
- Performance characteristics
- Security model overview
- Database schema
- Troubleshooting checklist
- Quick start guide

**Best for:** Fast reference, troubleshooting, quick lookups

---

## Quick Summary

### What is this?
S3 integration for PostgreSQL database backup management. Supports any S3-compatible object storage (AWS S3, Hetzner Object Storage, MinIO, etc.).

### Key Components

1. **Library**: `aws-sdk-s3` v1.37.0 with rustls
2. **Main Implementation**: `/backend/src/services/maintenance.rs`
3. **Configuration**: `configdb.json` + `contragent_settings` database table
4. **API**: GraphQL mutations and queries
5. **Backup Format**: PostgreSQL custom format (`.dump` files)

### Supported Providers
- AWS S3
- Hetzner Object Storage
- MinIO
- Any S3-compatible API

**Not Supported:** Mega.nz, Google Cloud Storage, Azure Blob Storage

### Core Functionality

#### Upload
```
pg_dump → {file}.dump → Upload to S3 → Delete local → Return metadata
```

#### Download
```
S3 GET → /tmp/{uuid}.dump → pg_restore → Delete temp
```

#### List
```
Local backups + S3 backups → Merge → Sort by date → Return 5 most recent
```

---

## Key File Paths

| Purpose | Path |
|---------|------|
| Main S3 Code | `/backend/src/services/maintenance.rs` |
| Config Structures | `/backend/src/config.rs` |
| GraphQL API | `/backend/src/graphql/maintenance_resolver.rs` |
| Settings Entity | `/backend/src/entities/contragent_setting.rs` |
| Configuration File | `/configdb.json` or `/configdb.example.json` |
| Dependencies | `/backend/Cargo.toml` (line 30) |

---

## Configuration Example

### Enable S3 in configdb.json
```json
"object_storage": {
  "enabled": true,
  "endpoint": "https://eu-central.s3.hetzner.cloud",
  "access_key": "YOUR_KEY",
  "secret_key": "YOUR_SECRET",
  "region": "eu-central",
  "bucket": "your-bucket",
  "prefix": "rs-ac-bg/backups",
  "force_path_style": true
}
```

---

## GraphQL Examples

### Create Backup
```graphql
mutation {
  createDatabaseBackup {
    backup { fileName sizePretty storageType }
    durationMs
    remoteObjectKey
  }
}
```

### Check Status
```graphql
query {
  databaseMaintenanceStatus {
    databaseSizePretty
    objectStorageEnabled
    recentBackups { fileName storageType objectKey }
  }
}
```

### Update Settings
```graphql
mutation {
  updateObjectStorageSettings(input: {
    enabled: true
    endpoint: "https://endpoint"
    accessKey: "key"
    secretKey: "secret"
    region: "region"
    bucket: "bucket"
    prefix: "prefix"
    forcePathStyle: true
  }) {
    enabled
    endpoint
  }
}
```

### Restore
```graphql
mutation {
  restoreDatabase(input: {
    storageType: ObjectStorage
    identifier: "rs-ac-bg/backups/rs_ac_bg_20240101_143022.dump"
  }) {
    source
    storageType
    durationMs
  }
}
```

---

## Configuration Persistence

### Dual-Layer Configuration
1. **Static Layer**: `configdb.json` (loaded on startup)
2. **Runtime Layer**: `contragent_settings` table (database)
3. **Merge**: Runtime settings override static settings

### Database Storage Keys
- `maintenance.object_storage.enabled`
- `maintenance.object_storage.endpoint`
- `maintenance.object_storage.access_key`
- `maintenance.object_storage.secret_key` (encrypted=true)
- `maintenance.object_storage.region`
- `maintenance.object_storage.bucket`
- `maintenance.object_storage.prefix`
- `maintenance.object_storage.force_path_style`

---

## Error Handling

### Graceful Degradation
- Storage disabled → Skip upload (OK)
- Missing credentials → Skip upload (OK)
- Empty bucket → Skip upload (OK)
- Network error → Propagate (ERROR)
- File error → Propagate (ERROR)

### Error Messages (Bulgarian)
- "Неуспешно четене на архивния файл" = File read failed
- "Object storage не е конфигуриран" = Not configured
- "Не е зададен bucket" = Bucket not configured

---

## Security

### Credentials
- Stored in database as encrypted entries
- Never logged in error messages
- Secret key only marked as boolean in API responses
- Validated before use

### Permissions
- Requires database access (GraphQL context)
- Settings shared across all companies
- Per-backup access control not implemented

### Data Protection
- Temporary restore files use UUID naming
- Automatic cleanup after restore
- Path-style URLs for non-AWS compatibility

---

## Potential Enhancements

1. Backup compression before upload
2. At-rest encryption for backups
3. Multipart upload for large files
4. S3 versioning support
5. Lifecycle policies for auto-expiration
6. Connection pooling for performance
7. CloudWatch/metrics monitoring
8. Per-company settings (currently global)
9. Audit logging for backup operations
10. Mega.nz support (would need separate SDK)

---

## Troubleshooting

| Problem | Check |
|---------|-------|
| Upload fails | Credentials, bucket, endpoint, network |
| Download fails | Object exists, credentials, network |
| Settings won't save | Database connection, table permissions |
| Timeout errors | Endpoint URL, firewall, network |
| Auth fails | Access key/secret, region mismatch |
| Empty list | Bucket name, prefix, .dump filter |

---

## Related Technologies

### PostgreSQL Utilities
- `pg_dump`: Database export (used for backup)
- `pg_restore`: Database import (used for restore)
- Custom format: Binary backup format

### AWS SDK
- `aws-sdk-s3`: Official Rust SDK v1.37.0
- `aws-smithy-*`: SDK runtime and type definitions
- `rustls`: TLS implementation

### Application Stack
- `sea-orm`: Database ORM (settings storage)
- `async-graphql`: GraphQL framework (API)
- `tokio`: Async runtime (concurrent operations)
- `chrono`: Date/time handling (timestamps)
- `uuid`: Unique ID generation (temp files)

---

## Getting Started

### 1. Review the Files
Start with the **Quick Reference** for overview, then dive into specific documents:
- First time? Read S3_QUICK_REFERENCE.md
- Need details? Read S3_IMPLEMENTATION_ANALYSIS.md
- Looking for code? Read S3_CODE_SNIPPETS.md

### 2. Enable in configdb.json
Set `"enabled": true` and fill in your S3 credentials.

### 3. Test
Create a backup via GraphQL mutation, verify it uploaded via query.

### 4. Restore
Use the restore mutation with remote storage type to verify download works.

---

## Document Version

- **Created**: 2024-11-05
- **Analyzed Codebase**: rs-ac-bg (Rust Accounting Backend)
- **S3 Library**: aws-sdk-s3 v1.37.0
- **Target Providers**: AWS S3, Hetzner, MinIO, S3-compatible services

---

## Note

This analysis was performed on the application as it exists in the repository. All line numbers, file paths, and code snippets are accurate as of the analysis date. For the most current implementation, refer directly to the source files in the repository.

