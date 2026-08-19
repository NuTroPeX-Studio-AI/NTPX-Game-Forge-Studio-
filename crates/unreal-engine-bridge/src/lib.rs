//! Unreal Engine bridge contracts for NTPX GameForge Ω∞.
//!
//! Shared orchestration remains engine-neutral; this crate is the explicit Unreal adapter boundary.
//! It constructs validated commands and asset/package intents but does not launch Unreal itself.

use std::{error::Error, fmt};

use serde::{Deserialize, Serialize};

/// Supported host classes for Unreal tooling.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HostPlatform {
    Windows,
    Linux,
}

/// Discovered Unreal installation metadata.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnrealInstallation {
    pub version: String,
    pub root: String,
    pub host: HostPlatform,
}

/// Unreal command family.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UnrealCommandKind {
    UnrealBuildTool,
    AutomationTool,
    EditorCommandlet,
    Editor,
}

/// Validated command intent for a later governed process driver.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnrealCommand {
    pub executable: String,
    pub args: Vec<String>,
    pub kind: UnrealCommandKind,
}

/// Asset import intent.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssetImportRequest {
    pub source_path: String,
    pub destination_path: String,
    pub replace_existing: bool,
}

/// Package request for a target platform/configuration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackageRequest {
    pub project_path: String,
    pub target_platform: String,
    pub configuration: String,
    pub output_directory: String,
}

/// Unreal host-specific path driver.
pub trait UnrealHostDriver: Send + Sync {
    fn host(&self) -> HostPlatform;
    fn executable_for(
        &self,
        installation: &UnrealInstallation,
        kind: UnrealCommandKind,
    ) -> Result<String, UnrealBridgeError>;
}

/// Windows Unreal path driver.
#[derive(Debug, Clone, Copy, Default)]
pub struct WindowsUnrealDriver;

/// Linux Unreal path driver.
#[derive(Debug, Clone, Copy, Default)]
pub struct LinuxUnrealDriver;

impl UnrealHostDriver for WindowsUnrealDriver {
    fn host(&self) -> HostPlatform {
        HostPlatform::Windows
    }

    fn executable_for(
        &self,
        installation: &UnrealInstallation,
        kind: UnrealCommandKind,
    ) -> Result<String, UnrealBridgeError> {
        ensure_host(installation, self.host())?;
        let relative = match kind {
            UnrealCommandKind::UnrealBuildTool => {
                "Engine/Binaries/DotNET/UnrealBuildTool/UnrealBuildTool.exe"
            }
            UnrealCommandKind::AutomationTool => "Engine/Build/BatchFiles/RunUAT.bat",
            UnrealCommandKind::EditorCommandlet | UnrealCommandKind::Editor => {
                "Engine/Binaries/Win64/UnrealEditor.exe"
            }
        };
        Ok(join_root(&installation.root, relative))
    }
}

impl UnrealHostDriver for LinuxUnrealDriver {
    fn host(&self) -> HostPlatform {
        HostPlatform::Linux
    }

    fn executable_for(
        &self,
        installation: &UnrealInstallation,
        kind: UnrealCommandKind,
    ) -> Result<String, UnrealBridgeError> {
        ensure_host(installation, self.host())?;
        let relative = match kind {
            UnrealCommandKind::UnrealBuildTool => {
                "Engine/Binaries/DotNET/UnrealBuildTool/UnrealBuildTool"
            }
            UnrealCommandKind::AutomationTool => "Engine/Build/BatchFiles/RunUAT.sh",
            UnrealCommandKind::EditorCommandlet | UnrealCommandKind::Editor => {
                "Engine/Binaries/Linux/UnrealEditor"
            }
        };
        Ok(join_root(&installation.root, relative))
    }
}

fn ensure_host(
    installation: &UnrealInstallation,
    expected: HostPlatform,
) -> Result<(), UnrealBridgeError> {
    if installation.root.trim().is_empty() {
        return Err(UnrealBridgeError::EmptyInstallationRoot);
    }
    if installation.version.trim().is_empty() {
        return Err(UnrealBridgeError::EmptyVersion);
    }
    if installation.host != expected {
        return Err(UnrealBridgeError::HostMismatch {
            expected,
            actual: installation.host,
        });
    }
    Ok(())
}

fn join_root(root: &str, relative: &str) -> String {
    format!("{}/{}", root.trim_end_matches(['/', '\\']), relative)
}

/// Unreal bridge validation failures.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UnrealBridgeError {
    EmptyInstallationRoot,
    EmptyVersion,
    HostMismatch {
        expected: HostPlatform,
        actual: HostPlatform,
    },
    EmptyProjectPath,
    EmptyCommandArgument,
    EmptyAssetPath,
    EmptyPackageField,
}

impl fmt::Display for UnrealBridgeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyInstallationRoot => {
                formatter.write_str("Unreal installation root must not be empty")
            }
            Self::EmptyVersion => formatter.write_str("Unreal version must not be empty"),
            Self::HostMismatch { expected, actual } => {
                write!(
                    formatter,
                    "Unreal host mismatch: expected {expected:?}, found {actual:?}"
                )
            }
            Self::EmptyProjectPath => formatter.write_str("uproject path must not be empty"),
            Self::EmptyCommandArgument => {
                formatter.write_str("Unreal command arguments must not be empty strings")
            }
            Self::EmptyAssetPath => formatter.write_str("asset import paths must not be empty"),
            Self::EmptyPackageField => {
                formatter.write_str("package request fields must not be empty")
            }
        }
    }
}

impl Error for UnrealBridgeError {}

/// Build a validated Unreal command intent for later governor-gated execution.
///
/// # Errors
/// Returns [`UnrealBridgeError`] for host mismatches, malformed installation data, empty project
/// paths, or empty arguments.
pub fn build_command(
    driver: &dyn UnrealHostDriver,
    installation: &UnrealInstallation,
    kind: UnrealCommandKind,
    project_path: &str,
    args: Vec<String>,
) -> Result<UnrealCommand, UnrealBridgeError> {
    if project_path.trim().is_empty() {
        return Err(UnrealBridgeError::EmptyProjectPath);
    }
    if args.iter().any(|arg| arg.trim().is_empty()) {
        return Err(UnrealBridgeError::EmptyCommandArgument);
    }
    let executable = driver.executable_for(installation, kind)?;
    let mut complete_args = Vec::with_capacity(args.len() + 1);
    complete_args.push(project_path.to_owned());
    complete_args.extend(args);
    Ok(UnrealCommand {
        executable,
        args: complete_args,
        kind,
    })
}

/// Validate an asset import request before it reaches an engine process.
///
/// # Errors
/// Returns [`UnrealBridgeError`] when either path is empty.
pub fn validate_asset_import(request: &AssetImportRequest) -> Result<(), UnrealBridgeError> {
    if request.source_path.trim().is_empty() || request.destination_path.trim().is_empty() {
        return Err(UnrealBridgeError::EmptyAssetPath);
    }
    Ok(())
}

/// Validate a package request before UAT dispatch.
///
/// # Errors
/// Returns [`UnrealBridgeError`] when any required field is empty.
pub fn validate_package_request(request: &PackageRequest) -> Result<(), UnrealBridgeError> {
    let required = [
        request.project_path.as_str(),
        request.target_platform.as_str(),
        request.configuration.as_str(),
        request.output_directory.as_str(),
    ];
    if required.iter().any(|value| value.trim().is_empty()) {
        return Err(UnrealBridgeError::EmptyPackageField);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn windows_driver_builds_ubt_command() {
        let installation = UnrealInstallation {
            version: "5.x".to_owned(),
            root: "C:/UE".to_owned(),
            host: HostPlatform::Windows,
        };
        let command = build_command(
            &WindowsUnrealDriver,
            &installation,
            UnrealCommandKind::UnrealBuildTool,
            "C:/Project/Game.uproject",
            vec!["Development".to_owned()],
        )
        .expect("valid command");
        assert!(command.executable.ends_with("UnrealBuildTool.exe"));
        assert_eq!(command.args[0], "C:/Project/Game.uproject");
    }

    #[test]
    fn host_substitution_fails_closed() {
        let installation = UnrealInstallation {
            version: "5.x".to_owned(),
            root: "/opt/Unreal".to_owned(),
            host: HostPlatform::Linux,
        };
        let error = WindowsUnrealDriver
            .executable_for(&installation, UnrealCommandKind::Editor)
            .expect_err("wrong host must fail");
        assert!(matches!(error, UnrealBridgeError::HostMismatch { .. }));
    }

    #[test]
    fn package_request_requires_all_fields() {
        let request = PackageRequest {
            project_path: "Game.uproject".to_owned(),
            target_platform: String::new(),
            configuration: "Shipping".to_owned(),
            output_directory: "dist".to_owned(),
        };
        assert_eq!(
            validate_package_request(&request),
            Err(UnrealBridgeError::EmptyPackageField)
        );
    }
}
