# thag 0.2 – Rust REPL and script runner with dependency inference

Rust is great, but creating a full Cargo project for every little experiment gets old. I built **thag**, a REPL and script runner that lets you run snippets, scripts, and full projects seamlessly — now at version **0.2**.

I’ve long been a fan of the [cargo-script initiative](https://internals.rust-lang.org/t/pre-rfc-cargo-script-for-everyone/18639), but in the meantime I wanted a fast, low-boilerplate way to slice and dice Rust while keeping compatibility with existing tooling.

[**thag 0.2**](https://github.com/durbanlegend/thag_rs/blob/main/README.md) brings theming goodness and a companion [profiler](https://github.com/durbanlegend/thag_rs/blob/main/thag_profiler/README.md) for good measure.

---

## 🦀 What is thag?

`thag` (crate name `thag_rs`) is a Rust playground and REPL that lowers the barriers to running quick Rust experiments while still supporting full project complexity when needed.

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

## 📦 More info

- GitHub: [thag_rs](https://github.com/durbanlegend/thag_rs)
- Crates.io: [thag_rs](https://crates.io/crates/thag_rs)
- Profiler: [thag_profiler](https://github.com/durbanlegend/thag_rs/blob/main/thag_profiler/README.md)

---

Feedback, ideas, and performance impressions are very welcome — especially from anyone who’s used **cargo-script** or **evcxr**.
