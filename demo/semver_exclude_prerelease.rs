/// Prototype of excluding pre-release crates from cargo queries.
//# Purpose: Prototype technique for `thag_rs`.
//# Categories: prototype, technique
use semver::{Version, VersionReq};

fn matches_version_without_prerelease(version_req: &VersionReq, version: &Version) -> bool {
    // Ensure the version matches the requirement and is not a pre-release
    version_req.matches(version) && version.pre.is_empty()
}

fn main() {
    let req = VersionReq::parse(">=1.0.0").expect("Invalid version requirement");

    let versions = vec![
        Version::parse("1.0.0").unwrap(),
        Version::parse("1.1.0-alpha").unwrap(),
        Version::parse("1.2.0-beta.2").unwrap(),
        Version::parse("1.2.0").unwrap(),
    ];

    let valid_versions: Vec<_> = versions
        .into_iter()
        .filter(|v| matches_version_without_prerelease(&req, v))
        .collect();

    for version in valid_versions {
        println!("Valid version: {}", version);
    }
}
