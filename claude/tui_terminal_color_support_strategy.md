


In my `thag` script runner I've gone to quite a bit of trouble to support terminals/emulators without colour support, so that they won't be sent ANSI control strings that would appear on the terminal - on the off-chance that this will ever be required. However, when testing this with TUI editing using `ratatui`, it seems that in practice `ratatui` depends on highlighting, dimming etc. I'm not 100% sure. I only hit problems trying to apply this as conditional logic to a FileDialog TUI consisting of a file list and an input box and the need to be able to scroll between them and in some way highlight which one has the focus. Then I realised I was still relying on styling to show the current line, cursor position etc in the editor itself.

So I'm thinking of making at least Basic (16-colour) support as a condition for using the TUI editing facilities of the project. This does not include the REPL, which uses `reedline` and has a `with_ansi_colors(bool)` builder to suppress styling.

Firstly, is this a good idea or can I get it to work? I'm close to a solution apart from this issue.

Secondly, assuming it is a good idea, how best to implement it? My first thought was yet another feature, which would have to be included in the `tui` feature but not in the `repl` feature that currently includes `tui`. However I'm reluctant to add features as I think the feature dependency tree is already quite complex, e.g I currently need to run clippy against 9 separate feature sets to ensure that I don't inadvertently feature-gate any artifacts. Alternatively I might just hide the TUI functionality from such terminals, since it's easy to detect them at any point in the code: `TermAttributes::get_or_init().color_support == ColorSupport::None`.

Here's the feature dependency tree as it currently stands:
# Feature dependency tree:
default
└── full
    ├── repl
    │   └── tui
    │       ├── build
    │       │   ├── ast
    │       │   │   ├── core  ★                # Fundamental feature set
    │       │   │   │   ├── error_handling     # Error types and handling
    │       │   │   │   ├── log_impl           # Basic logging infrastructure
    │       │   │   │   │   └── (simplelog | env_logger)
    │       │   │   │   └── styling            # Basic terminal styling
    │       │   │   ├── quote
    │       │   │   └── syn
    │       │   ├── config
    │       │   │   ├── core  ★ (shared)       # Core features required here too
    │       │   │   ├── mockall
    │       │   │   ├── serde_with
    │       │   │   └── toml_edit
    │       │   └── crossterm                  # Terminal control
    │       │
    │       ├── ratatui                        # TUI framework
    │       ├── tui-textarea                   # Text editing widget
    │       ├── crokey                         # Keyboard handling
    │       ├── serde_json                     # JSON support
    │       └── scopeguard                     # Resource cleanup (shared with color_detect)
    │
    └── color_detect     # Optional terminal detection, only included in full
        ├── config
        ├── crossterm    # (shared with build)
        ├── scopeguard   # (shared with tui)
        ├── supports-color
        └── termbg


Any advice as to how to proceed?

Based on the context and the complexity of your feature dependency tree, I would recommend going with your second approach - hiding TUI functionality from terminals without color support rather than adding another feature flag. Here's why:

1. **Practical Reality**: As you've noticed, TUI interfaces generally depend on some level of color/styling support to be usable. Making this a hard requirement is reasonable and matches real-world usage patterns.

2. **Feature Complexity**: Your feature tree is already quite sophisticated. Adding another feature would increase complexity and testing burden without providing much practical benefit.

3. **Runtime Detection**: Since you can easily detect color support at runtime using `TermAttributes::get_or_init().color_support`, this provides a clean way to handle the situation:
   ```rust
   if TermAttributes::get_or_init().color_support == ColorSupport::None {
       // Either fall back to non-TUI mode or display an error message
       eprintln!("TUI features require terminal color support");
       // Handle accordingly...
   }
   ```

4. **User Experience**: It's more straightforward to communicate to users "TUI features require a terminal with at least basic color support" than to maintain a separate feature flag for this edge case.

Implementation suggestion:
```rust
pub fn launch_tui() -> Result<(), Error> {
    if TermAttributes::get_or_init().color_support == ColorSupport::None {
        return Err(Error::UnsupportedTerminal(
            "TUI features require terminal color support".into()
        ));
    }
    // Proceed with TUI initialization...
}
```

This approach keeps your code cleaner and feature dependencies manageable while still handling the edge case appropriately.
