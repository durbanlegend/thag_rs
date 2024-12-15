/*[toml]
[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
*/

use serde::{Deserialize, Serialize};
use serde_json;

#[derive(Serialize, Deserialize, Debug)]
struct MyData {
    values: Vec<i128>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a sample array of i128 values
    let data = MyData {
        values: vec![
            123456789012345678901234567890i128,
            -987654321098765432109876543210i128,
        ],
    };

    // Serialize to a JSON string
    let json_string = serde_json::to_string(&data)?;
    println!("Serialized JSON: {}", json_string);

    // Deserialize back to MyData
    let deserialized_data: MyData = serde_json::from_str(&json_string)?;
    println!("Deserialized struct: {:?}", deserialized_data);

    Ok(())
}
