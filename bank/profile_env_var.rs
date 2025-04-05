use std::env;
use std::fmt::{Display, Formatter};
use std::path::PathBuf;
use std::str::FromStr;

// Define the Profile Type enum
#[derive(Debug, Clone, PartialEq)]
pub enum ProfileType {
    Time,
    Memory,
    Both,
}

// Implement FromStr for ProfileType
impl FromStr for ProfileType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().as_str() {
            "time" => Ok(ProfileType::Time),
            "memory" => Ok(ProfileType::Memory),
            "both" => Ok(ProfileType::Both),
            _ => Err(format!(
                "Invalid profile type '{}'. Expected 'Time', 'Memory', or 'Both'",
                s
            )),
        }
    }
}

// Define the DebugLog enum
#[derive(Debug, Clone, PartialEq)]
pub enum DebugLog {
    None,
    Quiet,
    Announce,
}

// Implement FromStr for DebugLog
impl FromStr for DebugLog {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().as_str() {
            "none" => Ok(DebugLog::None),
            "quiet" => Ok(DebugLog::Quiet),
            "announce" => Ok(DebugLog::Announce),
            _ => Err(format!(
                "Invalid debug log type '{}'. Expected 'None', 'Quiet', or 'Announce'",
                s
            )),
        }
    }
}

// Define our Configuration struct
#[derive(Debug)]
struct ProfileConfig {
    enabled: bool,
    profile_type: ProfileType,
    output_dir: PathBuf,
    debug_log: DebugLog,
    detailed_memory: bool,
}

impl Display for ProfileConfig {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Profile Config:\n")?;
        write!(f, "  Enabled: {}\n", self.enabled)?;
        write!(f, "  Profile Type: {:?}\n", self.profile_type)?;
        write!(f, "  Output Directory: {}\n", self.output_dir.display())?;
        write!(f, "  Debug Log: {:?}\n", self.debug_log)?;
        write!(f, "  Detailed Memory: {}", self.detailed_memory)
    }
}

fn parse_profile_config() -> Result<ProfileConfig, Vec<String>> {
    let env_var = match env::var("THAG_PROFILE") {
        Ok(val) => val,
        Err(_) => return Err(vec!["THAG_PROFILE environment variable not set".to_string()]),
    };

    let parts: Vec<&str> = env_var.split(',').collect();
    let mut errors = Vec::new();

    // Parse enabled flag
    let enabled = if parts.get(0).map_or("", |s| *s).trim().is_empty() {
        errors
            .push("First element (enabled flag) is empty. Expected 'true' or 'false'".to_string());
        false
    } else {
        match parts.get(0).unwrap().trim().parse::<bool>() {
            Ok(val) => val,
            Err(_) => {
                errors.push(format!(
                    "Failed to parse '{}' as boolean for enabled flag. Expected 'true' or 'false'",
                    parts.get(0).unwrap()
                ));
                false
            }
        }
    };

    // Parse profile type
    let profile_type = if parts.get(1).map_or("", |s| *s).trim().is_empty() {
        errors.push(
            "Second element (profile type) is empty. Expected 'Time', 'Memory', or 'Both'"
                .to_string(),
        );
        ProfileType::Time // Default
    } else {
        match parts.get(1).unwrap().parse::<ProfileType>() {
            Ok(val) => val,
            Err(e) => {
                errors.push(e);
                ProfileType::Time // Default
            }
        }
    };

    // Parse output directory
    let output_dir = if parts.get(2).map_or("", |s| *s).trim().is_empty() {
        PathBuf::from(".") // Default
    } else {
        PathBuf::from(parts.get(2).unwrap().trim())
    };

    // Parse debug log
    let debug_log = if parts.get(3).map_or("", |s| *s).trim().is_empty() {
        errors.push(
            "Fourth element (debug log) is empty. Expected 'None', 'Quiet', or 'Announce'"
                .to_string(),
        );
        DebugLog::None // Default
    } else {
        match parts.get(3).unwrap().parse::<DebugLog>() {
            Ok(val) => val,
            Err(e) => {
                errors.push(e);
                DebugLog::None // Default
            }
        }
    };

    // Parse detailed memory
    let detailed_memory = if let Some(val) = parts.get(4) {
        if val.trim().is_empty() {
            false // Default if empty
        } else {
            match val.trim().parse::<bool>() {
                Ok(val) => {
                    // Validate that detailed memory is only true for Memory or Both profile types
                    if val && profile_type == ProfileType::Time {
                        errors.push(
                            "Detailed memory profiling can only be enabled with ProfileType::Memory or ProfileType::Both"
                                .to_string(),
                        );
                        false
                    } else {
                        val
                    }
                }
                Err(_) => {
                    errors.push(format!(
                        "Failed to parse '{}' as boolean for detailed memory flag. Expected 'true' or 'false'",
                        val
                    ));
                    false
                }
            }
        }
    } else {
        false // Default if not present
    };

    if !errors.is_empty() {
        return Err(errors);
    }

    Ok(ProfileConfig {
        enabled,
        profile_type,
        output_dir,
        debug_log,
        detailed_memory,
    })
}

fn format_usage_instructions() -> String {
    "
THAG_PROFILE environment variable format:
    THAG_PROFILE=<enabled>,<profile_type>,<output_dir>,<debug_log>,<detailed_memory>

Where:
    <enabled>: true or false
    <profile_type>: time, memory, or both
    <output_dir>: Directory path where profiling results will be stored (defaults to '.')
    <debug_log>: none, quiet, or announce
    <detailed_memory>: true or false (only valid with Memory or Both profile types)

Example:
    THAG_PROFILE=true,both,./profiles,quiet,true
"
    .to_string()
}

fn main() {
    match parse_profile_config() {
        Ok(config) => {
            println!("Successfully parsed profile configuration:");
            println!("{}", config);
            // Use config in your application...
        }
        Err(errors) => {
            println!("Error parsing THAG_PROFILE environment variable:");
            for error in &errors {
                println!("  - {}", error);
            }
            println!("{}", format_usage_instructions());
        }
    }
}
