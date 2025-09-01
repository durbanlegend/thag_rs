# Terminal Palette Querying with OSC 4

This directory contains demonstration scripts for querying terminal color palettes using OSC (Operating System Command) sequences, specifically OSC 4 for palette interrogation.

## Scripts Overview

### `query_terminal_palette.rs` - Safe Demonstration
A comprehensive educational script that demonstrates the concepts of OSC 4 palette querying with:
- ✅ Safe mock implementation showing the parsing logic
- ✅ Educational explanations of OSC sequences
- ✅ Comparison with current thag themes
- ✅ Practical alternative detection methods
- ✅ No risk of terminal interference

**Recommended for:** Learning, understanding concepts, safe exploration

### `experimental_palette_query.rs` - Real Implementation
An experimental script that attempts actual OSC 4 queries:
- ⚠️ Real terminal I/O operations
- ⚠️ May cause terminal artifacts or flickering
- ⚠️ Platform-specific (Unix/Linux focus)
- ⚠️ Success varies by terminal emulator

**Recommended for:** Advanced users, testing real implementations

## OSC 4 Sequence Format

### Query Format
```
\x1b]4;<index>;?\x07
```
- `\x1b]` - OSC introducer
- `4` - Palette command
- `<index>` - Color index (0-15 for ANSI palette)
- `?` - Query marker
- `\x07` - BEL terminator

### Response Format
```
\x1b]4;<index>;rgb:<r>/<g>/<b>\x07
```
Example: `\x1b]4;1;rgb:ff00/0000/8000\x07` means Color 1 = RGB(255, 0, 128)

Alternative format: `\x1b]4;<index>;#RRGGBB\x07`

## Terminal Support

### Well Supported
- ✅ **WezTerm** - Excellent OSC 4 support
- ✅ **Alacritty** - Full OSC sequence support
- ✅ **iTerm2** - Complete implementation
- ✅ **Kitty** - Good OSC support
- ✅ **Windows Terminal** (1.22+) - Recent OSC 4 support
- ✅ **GNOME Terminal** - Modern versions
- ✅ **Konsole** - KDE's terminal

### Limited Support
- ⚠️ **tmux/screen** - May need passthrough configuration
- ⚠️ **SSH sessions** - Depends on terminal forwarding
- ⚠️ **IDE terminals** - Often filtered or blocked

### Not Supported
- ❌ **Emacs terminal** - No OSC support
- ❌ **Basic/legacy terminals** - Limited escape sequence support

## Implementation Challenges

1. **Input Parsing**: OSC responses come via terminal input, not stdin events
2. **Timing**: Responses can be delayed or lost
3. **Format Variations**: Different terminals use slightly different formats
4. **Raw Mode**: Requires direct terminal access, not event-based I/O
5. **Platform Differences**: Unix/Windows have different terminal APIs

## Usage Examples

### Safe Learning (Recommended)
```bash
cargo run demo/query_terminal_palette.rs
```

### Experimental Testing (Advanced)
```bash
# Only in native terminals, not IDE embedded ones
cargo run demo/experimental_palette_query.rs
```

## Practical Alternatives

For production applications, consider these more reliable methods:

1. **Environment Variables**: `TERM`, `COLORTERM`, `TERM_PROGRAM`
2. **Background Detection**: Using libraries like `termbg`
3. **Color Support Detection**: Terminal capability probing
4. **User Configuration**: Let users specify their preferences
5. **Theme Synchronization**: Use OSC sequences to *set* colors instead

## Integration with thag_styling

The palette querying functionality integrates with thag_styling's existing capabilities:

- **Theme Comparison**: Compare queried colors with current theme
- **Palette Sync**: Use results to improve `PaletteSync` accuracy  
- **Color Mapping**: Better ANSI color role assignments
- **Terminal Detection**: Enhanced terminal capability detection

## Code Structure

### Core Components

- `Rgb` struct for color representation
- `PaletteError` enum for error handling
- Parsing functions for OSC 4 responses
- Query functions for individual colors
- Display functions for visualization

### Key Functions

- `parse_osc4_response()` - Parse terminal responses
- `parse_hex_component()` - Handle 2/4 digit hex values
- `query_palette_color_*()` - Various query implementations
- `display_palette_colors()` - Visual color representation
- `compare_with_thag_theme()` - Theme comparison

## Technical Notes

### Why is this challenging?

OSC sequences work at a lower level than typical terminal I/O:

1. **Not stdin events**: Responses bypass normal input processing
2. **Raw terminal access**: Need direct `/dev/tty` or equivalent
3. **Timing sensitive**: Terminal may delay or batch responses  
4. **Format variations**: No strict standard for response format
5. **Terminal multiplexers**: Screen/tmux can interfere

### Future Improvements

Potential enhancements for a production implementation:

- [ ] Windows-specific terminal I/O
- [ ] Better timeout handling
- [ ] Response caching
- [ ] Async query batching  
- [ ] Terminal-specific format handling
- [ ] Integration with `crossterm`'s raw mode
- [ ] Fallback to alternative detection methods

## Related Files

- `thag_styling/src/palette_sync.rs` - OSC sequence generation
- `demo/termbg.rs` - Background color detection
- `demo/terminal_palette_display.rs` - Color visualization
- `thag_styling/src/styling.rs` - Core styling functionality

## References

- [OSC Sequences Specification](https://invisible-island.net/xterm/ctlseqs/ctlseqs.html#h3-Operating-System-Commands)
- [Terminal Color Queries](https://github.com/crossterm-rs/crossterm/discussions)
- [termbg crate documentation](https://docs.rs/termbg/)
- [XTerm Color Operations](https://www.xfree86.org/current/ctlseqs.html)

---

*This functionality extends thag_styling's terminal integration capabilities and demonstrates advanced terminal programming concepts.*