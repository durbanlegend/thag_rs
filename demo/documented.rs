/// Published example from the `documented` crate.
//# Purpose: Explore making docs available at runtime.
//# Categories: crates, exploration, technique
use documented::{Documented, DocumentedFields, DocumentedVariants};

/// Trying is the first step to failure.
#[derive(Documented, DocumentedFields, DocumentedVariants)]
enum AlwaysPlay {
    /// And Kb8.
    #[allow(dead_code)]
    Kb1,
    /// But only if you are white.
    F6,
}

// Documented
assert_eq!(AlwaysPlay::DOCS, "Trying is the first step to failure.");

// DocumentedFields
assert_eq!(
    AlwaysPlay::FIELD_DOCS,
    ["And Kb8.", "But only if you are white."]
);
assert_eq!(AlwaysPlay::get_field_docs("Kb1"), Ok("And Kb8."));

// DocumentedVariants
assert_eq!(
    AlwaysPlay::F6.get_variant_docs(),
    "But only if you are white."
);
