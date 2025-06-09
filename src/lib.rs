use schemars::JsonSchema;
use serde::Deserialize;
use std::{fs, path};
use zed::ContextServerId;
use zed::settings::ContextServerSettings;
use zed_extension_api::{
    self as zed, ContextServerConfiguration, EnvVars, Project, Result, serde_json,
};

const PACKAGE_NAME: &str = "@azure/mcp";

struct AzureContextExtension {
    cached_context_server_path: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct AzureContextServerSettings {
    enable_production_credentials: bool,
}

impl AzureContextExtension {
    fn context_server_path(&mut self) -> Result<String> {
        if let Some(path) = &self.cached_context_server_path {
            if fs::metadata(&path).map_or(false, |stat| stat.is_file()) {
                return Ok(path.clone());
            }
        }

        let release = zed::latest_github_release(
            "Azure/azure-mcp",
            zed::GithubReleaseOptions {
                require_assets: true,
                pre_release: false,
            },
        )?;

        let (platform, arch) = zed::current_platform();
        let asset = release
            .assets
            .iter()
            .find(|asset| {
                asset.name
                    == format!(
                        "azure-mcp-{os}-{arch}-{version}.tgz",
                        os = match platform {
                            zed::Os::Linux => "linux",
                            zed::Os::Mac => "darwin",
                            zed::Os::Windows => "win32",
                        },
                        arch = match arch {
                            zed::Architecture::Aarch64 => "arm64",
                            zed::Architecture::X8664 => "x64",
                            zed::Architecture::X86 => "x86",
                        },
                        version = release.version,
                    )
            })
            .ok_or_else(|| format!("no matching asset found found"))?;

        let version_dir = format!("azure-mcp-{}", release.version);
        let context_server_path = match platform {
            zed::Os::Linux => format!("{}/package/dist/azmcp", version_dir),
            zed::Os::Mac => format!("{}/package/dist/azmcp", version_dir),
            zed::Os::Windows => format!("{}/package/dist/azmcp.exe", version_dir),
        };

        if !fs::metadata(&context_server_path).map_or(false, |stat| stat.is_file()) {
            // Download the asset
            zed::download_file(
                &asset.download_url,
                &version_dir,
                zed::DownloadedFileType::GzipTar,
            )
            .map_err(|err| format!("download error {}", err))?;

            // Cleanup old versions
            let entries =
                fs::read_dir(".").map_err(|e| format!("failed to list working directory {e}"))?;
            for entry in entries {
                let entry = entry.map_err(|e| format!("failed to load directory entry {e}"))?;
                if entry.file_name().to_str() != Some(&version_dir) {
                    fs::remove_dir_all(entry.path()).ok();
                }
            }
        }

        let abs_path = path::absolute(&context_server_path)
            .map_err(|e| format!("failed to get absolute path {e}"))?
            .to_str()
            .unwrap()
            .to_string();
        self.cached_context_server_path = Some(abs_path.clone());
        Ok(abs_path)
    }
}

impl zed::Extension for AzureContextExtension {
    fn new() -> Self {
        Self {
            cached_context_server_path: None,
        }
    }

    fn context_server_command(
        &mut self,
        _context_server_id: &ContextServerId,
        project: &zed::Project,
    ) -> Result<zed::Command> {
        // Parse settings
        let settings = ContextServerSettings::for_project("azure", project)?;
        let Some(settings) = settings.settings else {
            return Err("please configure the extension".into());
        };
        let settings: AzureContextServerSettings =
            serde_json::from_value(settings).map_err(|e| e.to_string())?;

        let mut env_config = EnvVars::new();
        if settings.enable_production_credentials {
            env_config.push(("AZURE_MCP_PRODUCTION_CREDENTIALS".into(), "true".into()));
        }

        // Download the Azure MCP server
        let latest_version = zed::npm_package_latest_version(PACKAGE_NAME)?;
        let version = zed::npm_package_installed_version(PACKAGE_NAME)?;
        if version.as_deref() != Some(latest_version.as_ref()) {
            zed::npm_install_package(PACKAGE_NAME, &latest_version)?;
        }

        Ok(zed::Command {
            command: self.context_server_path()?,
            args: vec!["server".to_string(), "start".to_string()],
            env: env_config,
        })
    }

    fn context_server_configuration(
        &mut self,
        _context_server_id: &ContextServerId,
        _project: &Project,
    ) -> Result<Option<ContextServerConfiguration>> {
        let installation_instructions =
            include_str!("../configuration/installation_instructions.md").to_string();
        let default_settings = include_str!("../configuration/default_settings.jsonc").to_string();
        let settings_schema =
            serde_json::to_string(&schemars::schema_for!(AzureContextServerSettings))
                .map_err(|e| e.to_string())?;

        Ok(Some(ContextServerConfiguration {
            installation_instructions,
            default_settings,
            settings_schema,
        }))
    }
}

zed::register_extension!(AzureContextExtension);
