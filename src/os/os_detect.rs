use std::error::Error;
use std::fmt;
use std::process::Command;

/// Custom error type for OS detection operations
#[derive(Debug)]
pub enum OsDetectorError {
    CommandFailed(String),
    UnsupportedOs(String),
    ParseError(String),
}

impl fmt::Display for OsDetectorError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            OsDetectorError::CommandFailed(msg) => write!(f, "Command execution failed: {}", msg),
            OsDetectorError::UnsupportedOs(msg) => write!(f, "Unsupported operating system: {}", msg),
            OsDetectorError::ParseError(msg) => write!(f, "Failed to parse system information: {}", msg),
        }
    }
}

impl Error for OsDetectorError {}

/// Represents detailed information about an operating system
#[derive(Debug, Clone)]
pub struct OsInfo {
    pub name: String,
    pub version: String,
    pub architecture: String,
    pub additional_info: Option<String>,
}

/// The main OS detection structure
#[derive(Debug, Default)]
pub struct OsDetector {
    cached_info: Option<OsInfo>,
}

impl OsDetector {
    /// Creates a new instance of OsDetector
    pub fn new() -> Self {
        Self { cached_info: None }
    }

    /// Retrieves operating system information
    ///
    /// # Returns
    /// Returns a Result containing OsInfo or an error
    ///
    /// # Examples
    /// ```rust
    /// let detector = OsDetector::new();
    /// match detector.get_os_info() {
    ///     Ok(info) => println!("OS: {}, Version: {}", info.name, info.version),
    ///     Err(e) => eprintln!("Error: {}", e),
    /// }
    /// ```
    pub fn get_os_info(&mut self) -> Result<OsInfo, OsDetectorError> {
        if let Some(info) = &self.cached_info {
            return Ok(info.clone());
        }

        let info = match std::env::consts::OS {
            "windows" => self.get_windows_info()?,
            "linux" => self.get_linux_info()?,
            "macos" => self.get_macos_info()?,
            os => return Err(OsDetectorError::UnsupportedOs(os.to_string())),
        };

        self.cached_info = Some(info.clone());
        Ok(info)
    }

    /// Retrieves Windows-specific information
    pub fn get_windows_info(&self) -> Result<OsInfo, OsDetectorError> {
        let output = Command::new("cmd")
            .args(["/C", "ver"])
            .output()
            .map_err(|e| OsDetectorError::CommandFailed(e.to_string()))?;

        let version_str = String::from_utf8_lossy(&output.stdout);
        let version = version_str
            .lines()
            .next()
            .ok_or_else(|| OsDetectorError::ParseError("Unable to read Windows version".to_string()))?
            .to_string();

        let arch = std::env::consts::ARCH.to_string();

        Ok(OsInfo {
            name: "Windows".to_string(),
            version,
            architecture: arch,
            additional_info: Some(self.get_windows_build_info()?),
        })
    }

    /// Retrieves Linux-specific information
    pub fn get_linux_info(&self) -> Result<OsInfo, OsDetectorError> {
        let os_release = std::fs::read_to_string("/etc/os-release")
            .map_err(|e| OsDetectorError::CommandFailed(e.to_string()))?;

        let mut name = String::new();
        let mut version = String::new();

        for line in os_release.lines() {
            if line.starts_with("NAME=") {
                name = line.split('=').nth(1).unwrap_or("Unknown")
                    .trim_matches('"').to_string();
            } else if line.starts_with("VERSION=") {
                version = line.split('=').nth(1).unwrap_or("Unknown")
                    .trim_matches('"').to_string();
            }
        }

        let kernel_version = Command::new("uname")
            .arg("-r")
            .output()
            .map_err(|e| OsDetectorError::CommandFailed(e.to_string()))?;

        Ok(OsInfo {
            name,
            version,
            architecture: std::env::consts::ARCH.to_string(),
            additional_info: Some(String::from_utf8_lossy(&kernel_version.stdout).trim().to_string()),
        })
    }

    /// Retrieves macOS-specific information
    pub fn get_macos_info(&self) -> Result<OsInfo, OsDetectorError> {
        let version = Command::new("sw_vers")
            .arg("-productVersion")
            .output()
            .map_err(|e| OsDetectorError::CommandFailed(e.to_string()))?;

        let build = Command::new("sw_vers")
            .arg("-buildVersion")
            .output()
            .map_err(|e| OsDetectorError::CommandFailed(e.to_string()))?;

        Ok(OsInfo {
            name: "macOS".to_string(),
            version: String::from_utf8_lossy(&version.stdout).trim().to_string(),
            architecture: std::env::consts::ARCH.to_string(),
            additional_info: Some(format!(
                "Build: {}",
                String::from_utf8_lossy(&build.stdout).trim()
            )),
        })
    }

    // Helper method to get Windows build information
    fn get_windows_build_info(&self) -> Result<String, OsDetectorError> {
        let output = Command::new("cmd")
            .args(["/C", "systeminfo"])
            .output()
            .map_err(|e| OsDetectorError::CommandFailed(e.to_string()))?;

        let info = String::from_utf8_lossy(&output.stdout);
        let build = info
            .lines()
            .find(|line| line.contains("OS Build"))
            .map(|line| line.split(':').nth(1).unwrap_or("").trim())
            .unwrap_or("Unknown");

        Ok(format!("Build: {}", build))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detector_creation() {
        let detector = OsDetector::new();
        assert!(detector.cached_info.is_none());
    }

    #[test]
    fn test_os_info_caching() {
        let mut detector = OsDetector::new();

        // First call should populate cache
        let first_result = detector.get_os_info();
        assert!(first_result.is_ok());
        assert!(detector.cached_info.is_some());

        // Second call should use cached value
        let second_result = detector.get_os_info();
        assert!(second_result.is_ok());

        // Results should be equal
        assert_eq!(
            format!("{:?}", first_result.unwrap()),
            format!("{:?}", second_result.unwrap())
        );
    }
}