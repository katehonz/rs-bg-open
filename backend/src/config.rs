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
    /// Parse DATABASE_URL in format: postgresql://user:pass@host:port/dbname
    fn parse_database_url(url: &str) -> Option<(String, u16, String, String, String)> {
        // Format: postgresql://user:pass@host:port/dbname
        let url = url.strip_prefix("postgresql://").or_else(|| url.strip_prefix("postgres://"))?;

        // Split user:pass@host:port/dbname
        let (user_pass, rest) = url.split_once('@')?;
        let (user, pass) = user_pass.split_once(':')?;

        // Split host:port/dbname
        let (host_port, dbname) = rest.split_once('/')?;

        let (host, port_str) = if let Some((h, p)) = host_port.split_once(':') {
            (h, p)
        } else {
            (host_port, "5432")
        };

        let port = port_str.parse::<u16>().ok()?;

        Some((
            host.to_string(),
            port,
            user.to_string(),
            pass.to_string(),
            dbname.to_string(),
        ))
    }

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
        if let Ok(mut config) = Self::from_file() {
            tracing::info!("Configuration loaded from file");

            // Override database config with DATABASE_URL if present
            if let Ok(database_url) = std::env::var("DATABASE_URL") {
                tracing::info!("Overriding database config with DATABASE_URL environment variable");
                if let Some(parsed) = Self::parse_database_url(&database_url) {
                    config.config_file.database.host = parsed.0;
                    config.config_file.database.port = parsed.1;
                    config.config_file.database.username = parsed.2;
                    config.config_file.database.password = parsed.3;
                    config.config_file.database.database = parsed.4;
                    config.database_url = database_url;
                }
            }

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

        // Parse DATABASE_URL to extract connection details
        let (db_host, db_port, db_user, db_pass, db_name) = Self::parse_database_url(&database_url)
            .unwrap_or_else(|| ("localhost".to_string(), 5432, "postgres".to_string(), "password".to_string(), "rs_ac_bg".to_string()));

        // Create default config file structure for backward compatibility
        let config_file = ConfigFile {
            database: DatabaseConfig {
                host: db_host,
                port: db_port,
                database: db_name,
                username: db_user,
                password: db_pass,
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
