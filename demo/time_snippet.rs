/*[toml]
[dependencies]
time = "0.1.25"
*/
// Uses an old version of the time crate so we need the toml block
time::now().rfc822z()
