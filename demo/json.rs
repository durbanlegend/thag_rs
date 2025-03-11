/*[toml]
[dependencies]
serde = "1.0.219"
serde_json = "1.0.132"
*/

/// Demo of deserialising JSON with the featured crates.
//# Purpose: Demo featured crates.
//# Categories: crates, technique
use serde::de::Deserialize;
use serde_json::Value;

println!(
    "{:#?}",
    serde_json::from_str::<Value>(
        r#"{
            "jsonrpc": "2.0",
            "method": "textDocument/completion",
            "params": {
                "textDocument": {
                    "uri": "file://Users/thag/projects/thag_rs/demo/see_what_zog_do.rs"
                },
                "position": {
                    "line": 15,
                    "character": 14
                }
            },
            "id": 1
            }
    "#
    )
    .unwrap()
);

println!(
    "{:#?}",
    serde_json::from_str::<Value>(
        "{\"id\":1,\"jsonrpc\":\"2.0\",\"method\":\"textDocument/completion\",\"params\":{\"position\":{\"character\":26,\"line\":171},\"textDocument\":{\"uri\":\"file:///Users/thag/projects/thag_rs/src/see_what_zog_do.rs\"}}}"
    )
    .unwrap()
);
