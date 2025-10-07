use serde::Deserialize;
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize, Clone)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub database: String,
    pub username: String,
    pub password: String,
    pub max_connections: Option<u32>,
    pub min_connections: Option<u32>,
    pub connect_timeout: Option<u64>,
    pub acquire_timeout: Option<u64>,
    pub idle_timeout: Option<u64>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub workers: Option<usize>,
    pub enable_cors: Option<bool>,
    pub cors_origins: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct LoggingConfig {
    pub level: String,
    pub file: Option<String>,
    pub stdout: Option<bool>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct InitialSetupConfig {
    pub create_demo_company: bool,
    pub demo_company_name: String,
    pub demo_company_eik: String,
    pub demo_company_vat: String,
    pub load_chart_of_accounts: bool,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ObjectStorageConfig {
    pub enabled: bool,
    pub endpoint: Option<String>,
    pub access_key: String,
    pub secret_key: String,
    pub region: String,
    pub bucket: String,
    pub prefix: Option<String>,
    pub force_path_style: Option<bool>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ConfigFile {
    pub database: DatabaseConfig,
    pub server: ServerConfig,
    pub logging: LoggingConfig,
    pub initial_setup: InitialSetupConfig,
    pub object_storage: Option<ObjectStorageConfig>,
}

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub host: String,
    pub port: u16,
    pub config_file: ConfigFile,
}

impl Config {
    pub fn from_file() -> anyhow::Result<Self> {
        // Try to load from configdb.json in the same directory as the binary
        let config_path = if Path::new("configdb.json").exists() {
            "configdb.json"
        } else if Path::new("../configdb.json").exists() {
            "../configdb.json"
        } else {
            // Fall back to example config
            tracing::warn!("configdb.json not found, using configdb.example.json");
            "configdb.example.json"
        };

        let config_str = fs::read_to_string(config_path)
            .map_err(|e| anyhow::anyhow!("Failed to read config file '{}': {}", config_path, e))?;

        let config_file: ConfigFile = serde_json::from_str(&config_str)
            .map_err(|e| anyhow::anyhow!("Failed to parse config file: {}", e))?;

        // Build database URL from config
        let database_url = format!(
            "postgres://{}:{}@{}:{}/{}",
            config_file.database.username,
            config_file.database.password,
            config_file.database.host,
            config_file.database.port,
            config_file.database.database
        );

        Ok(Self {
            database_url,
            host: config_file.server.host.clone(),
            port: config_file.server.port,
            config_file,
        })
    }

    pub fn from_env() -> anyhow::Result<Self> {
        // First try to load from file
        if let Ok(config) = Self::from_file() {
            tracing::info!("Configuration loaded from file");
            return Ok(config);
        }

        // Fall back to environment variables
        tracing::info!("Loading configuration from environment variables");

        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://postgres:password@localhost/rs_ac_bg".to_string());
        let host = std::env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
        let port = std::env::var("PORT")
            .unwrap_or_else(|_| "8080".to_string())
            .parse()?;

        // Create default config file structure for backward compatibility
        let config_file = ConfigFile {
            database: DatabaseConfig {
                host: "localhost".to_string(),
                port: 5432,
                database: "rs_ac_bg".to_string(),
                username: "postgres".to_string(),
                password: "password".to_string(),
                max_connections: Some(10),
                min_connections: Some(5),
                connect_timeout: Some(10),
                acquire_timeout: Some(10),
                idle_timeout: Some(600),
            },
            server: ServerConfig {
                host: host.clone(),
                port,
                workers: Some(4),
                enable_cors: Some(true),
                cors_origins: Some(vec![
                    "http://localhost:3000".to_string(),
                    "http://localhost:5173".to_string(),
                ]),
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                file: Some("logs/backend.log".to_string()),
                stdout: Some(true),
            },
            initial_setup: InitialSetupConfig {
                create_demo_company: true,
                demo_company_name: "Демо Фирма ООД".to_string(),
                demo_company_eik: "123456789".to_string(),
                demo_company_vat: "BG123456789".to_string(),
                load_chart_of_accounts: true,
            },
            object_storage: None,
        };

        Ok(Self {
            database_url,
            host,
            port,
            config_file,
        })
    }

    pub fn object_storage(&self) -> Option<&ObjectStorageConfig> {
        self.config_file
            .object_storage
            .as_ref()
            .filter(|cfg| cfg.enabled)
    }
}
