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

impl Display for ProfileType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ProfileType::Time => write!(f, "time"),
            ProfileType::Memory => write!(f, "memory"),
            ProfileType::Both => write!(f, "both"),
        }
    }
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
                "Invalid profile type '{}'. Expected 'time', 'memory', or 'both'",
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

impl Display for DebugLog {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            DebugLog::None => write!(f, "none"),
            DebugLog::Quiet => write!(f, "quiet"),
            DebugLog::Announce => write!(f, "announce"),
        }
    }
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
                "Invalid debug log type '{}'. Expected 'none', 'quiet', or 'announce'",
                s
            )),
        }
    }
}

// Define our Configuration struct with Option wrappers
#[derive(Debug)]
struct ProfileConfig {
    enabled: bool,
    profile_type: Option<ProfileType>,
    output_dir: Option<PathBuf>,
    debug_log: Option<DebugLog>,
    detailed_memory: bool,
}

impl Display for ProfileConfig {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Profile Config:\n")?;
        write!(f, "  Enabled: {}\n", self.enabled)?;

        match &self.profile_type {
            Some(pt) => write!(f, "  Profile Type: {}\n", pt)?,
            None => write!(f, "  Profile Type: none\n")?,
        }

        match &self.output_dir {
            Some(dir) => write!(f, "  Output Directory: {}\n", dir.display())?,
            None => write!(f, "  Output Directory: none\n")?,
        }

        match &self.debug_log {
            Some(log) => write!(f, "  Debug Log: {}\n", log)?,
            None => write!(f, "  Debug Log: none\n")?,
        }

        write!(f, "  Detailed Memory: {}", self.detailed_memory)
    }
}

fn parse_profile_config() -> Result<ProfileConfig, Vec<String>> {
    // If the env var is not present, return a default config with enabled=false
    let env_var = match env::var("THAG_PROFILER") {
        Ok(val) => val,
        Err(_) => {
            return Ok(ProfileConfig {
                enabled: false,
                profile_type: None,
                output_dir: None,
                debug_log: None,
                detailed_memory: false,
            });
        }
    };

    let parts: Vec<&str> = env_var.split(',').collect();
    let mut errors = Vec::new();

    // Parse profile type (first element now)
    let profile_type = if parts.get(0).map_or("", |s| *s).trim().is_empty() {
        errors.push(
            "First element (profile type) is empty. Expected 'time', 'memory', or 'both'"
                .to_string(),
        );
        None
    } else {
        match parts.get(0).unwrap().parse::<ProfileType>() {
            Ok(val) => Some(val),
            Err(e) => {
                errors.push(e);
                None
            }
        }
    };

    // Parse output directory (second element now)
    let output_dir = if parts.get(1).map_or("", |s| *s).trim().is_empty() {
        Some(PathBuf::from(".")) // Default to current directory if empty
    } else {
        Some(PathBuf::from(parts.get(1).unwrap().trim()))
    };

    // Parse debug log (third element now)
    let debug_log = if parts.get(2).map_or("", |s| *s).trim().is_empty() {
        errors.push(
            "Third element (debug log) is empty. Expected 'none', 'quiet', or 'announce'"
                .to_string(),
        );
        None
    } else {
        match parts.get(2).unwrap().parse::<DebugLog>() {
            Ok(val) => Some(val),
            Err(e) => {
                errors.push(e);
                None
            }
        }
    };

    // Parse detailed memory (fourth element now)
    let detailed_memory = if let Some(val) = parts.get(3) {
        if val.trim().is_empty() {
            false // Default if empty
        } else {
            match val.trim().parse::<bool>() {
                Ok(val) => {
                    // Validate that detailed memory is only true for Memory or Both profile types
                    if val
                        && profile_type
                            .as_ref()
                            .map_or(false, |pt| *pt == ProfileType::Time)
                    {
                        errors.push(
                            "Detailed memory profiling can only be enabled with profile_type=memory or profile_type=both"
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

    // If there are errors, return them
    if !errors.is_empty() {
        return Err(errors);
    }

    // All good, config is enabled
    Ok(ProfileConfig {
        enabled: true, // Assume enabled if all elements are valid
        profile_type,
        output_dir,
        debug_log,
        detailed_memory,
    })
}

fn format_usage_instructions() -> String {
    "
THAG_PROFILER environment variable format:
    THAG_PROFILER=<profile_type>,<output_dir>,<debug_log>,<detailed_memory>

Where:
    <profile_type>: time, memory, or both
    <output_dir>: Directory path where profiling results will be stored (defaults to '.')
    <debug_log>: none, quiet, or announce
    <detailed_memory>: true or false (only valid with memory or both profile types)

Example:
    THAG_PROFILER=both,./profiles,quiet,true
"
    .to_string()
}

fn main() {
    match parse_profile_config() {
        Ok(config) => {
            println!("Successfully parsed profile configuration:");
            println!("{}", config);

            if !config.enabled {
                println!("\nProfiling is not enabled. Set the THAG_PROFILER environment variable to enable it.");
                println!("{}", format_usage_instructions());
            }
            // Use config in your application...
        }
        Err(errors) => {
            println!("Error parsing THAG_PROFILER environment variable:");
            for error in &errors {
                println!("  - {}", error);
            }
            println!("{}", format_usage_instructions());
        }
    }
}
