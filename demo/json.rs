/*[toml]
[dependencies]
serde = "1.0.198"
serde_json = "1.0.116"
*/

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
                    "uri": "file://Users/donf/projects/rs-script/demo/fib_big_clap.rs"
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
        "{\"id\":1,\"jsonrpc\":\"2.0\",\"method\":\"textDocument/completion\",\"params\":{\"position\":{\"character\":26,\"line\":171},\"textDocument\":{\"uri\":\"file:///Users/donf/projects/rs-script/src/main.rs\"}}}"
    )
    .unwrap()
);
