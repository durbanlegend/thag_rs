# Announcing thag 0.2: A versatile Rust REPL/script runner with dependency inference

As a veteran experimenter, it's always struck me as unfortunate to have to make a new Rust project for every little thing. I'm a fan of the [cargo-script initiative](https://internals.rust-lang.org/t/pre-rfc-cargo-script-for-everyone/18639), but in the meantime I wanted a fast, low-boilerplate way to slice and dice Rust while keeping compatibility with existing tooling.

[**thag 0.2**](https://github.com/durbanlegend/thag_rs/blob/main/README.md) brings theming goodness and a companion [profiler](https://github.com/durbanlegend/thag_rs/blob/main/thag_profiler/README.md) for good measure.

---

## 🦀 What is thag?

thag (crate name thag_rs) is a Rust playground and REPL that aims to lower the barriers to running quick Rust experiments, while still supporting full project complexity when needed.

---

## 🎥 Demo

[**Watch the 7-minute demo on Asciinema**](https://asciinema.org/a/UXkWIf2gsFHD2JeCkLx5A60hG)
*(Recommended to watch in full-screen mode.)*

---

## ⚙️ Core features

- Run Rust programs, snippets, and expressions without explicit `Cargo.toml` boilerplate
- Automatic dependency inference, with configurable default-feature overrides
- Authentic `Cargo.toml` support for dependencies, features, profiles, and lints via embedded `/*[toml] ... */` blocks
- Built-in REPL with multi-line editing, history, TUI support, and preferred-editor integration
- Execute scripts from URLs for easy sharing
- Common engine behind CLI, REPL, stdin, and TUI modes
- Optional build commands as reusable binaries

---

## 🥕 Motivation

I need to work out ideas as snippets and save them for later. Prior script runners and the Rust Playground solve part of this, but I wanted:

  - Support for any and all dependencies.

  - The ability to run crate examples without cloning the crates

  - A tool that would be reliable, flexible, fast and frictionless.

  - Straightforward use of standard Rust / Cargo tooling for leverage and maintainability.

  - A minimum of manual dependency management - let the runner infer and build the Cargo.toml from the `use` statements, qualifiers etc. in the `syn` AST.

  - An AST- and cargo_toml-based engine so as to be reliable and not tripped up by code in comments.

  - Cross-platform capability and minimal restrictions on the development environment, so it could be useful to others.

  - A development path from idea to expression to snippet to script to module.

---

## 📦 More info

- GitHub: [thag_rs](https://github.com/durbanlegend/thag_rs)
- Crates.io: [thag_rs](https://crates.io/crates/thag_rs)
- Profiler: [thag_profiler](https://github.com/durbanlegend/thag_rs/blob/main/thag_profiler/README.md)
- Theming: [thag_styling](https://github.com/durbanlegend/thag_rs/blob/main/thag_styling/README.md)

---

Feedback, ideas, and performance impressions are very welcome — especially from anyone who’s used **cargo-script**, **rust-script**, **evcxr** or similar.
