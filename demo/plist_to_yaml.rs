/// Prototype of converting `plist` (`xml`) format to YAML.
/// There's no need to upload to an untrusted website or download an ancient Python
/// script that fails.
//# Purpose: Handy tool
//# Categories: technique, tools
//# Argument: INFILE:  Path to the input `plist` file. The extension may not necessarily be `plist`.
//# Argument: OUTFILE: Path to the output `YAML` file. The extensison would normally be `yaml` or 'yml`, but there are others that use `YAML` format.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: {} [--] INFILE OUTFILE", args[0]);
        std::process::exit(1);
    }

    // Parse plist into a serde_json::Value (or any serde-compatible type)
    let value: plist::Value = plist::Value::from_file(&args[1])?;

    // Convert plist::Value to a standard JSON Value / general structure
    let json_value: serde_json::Value = serde_json::to_value(&value)?;

    // Serialize to a YAML string
    let yaml_string = serde_yaml::to_string(&json_value)?;

    // Output or write to file
    // println!("{}", yaml_string);
    std::fs::write(&args[2], yaml_string)?;
    Ok(())
}
