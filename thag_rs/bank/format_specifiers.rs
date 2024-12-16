// Example with a loop:
let data = vec![
    ("short", "description 1"),
    ("very_long_key", "description 2"),
    ("medium", "description 3"),
];

let max_len = data.iter().map(|(k, _)| k.len()).max().unwrap_or(0);

for (key, desc) in data {
    println!("{:<width$} {}", key, desc, width=max_len + 2);
}
