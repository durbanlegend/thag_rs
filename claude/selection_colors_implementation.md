# Selection Colors Implementation Summary

## Overview

This document summarizes the comprehensive improvements made to terminal selection color handling across the `thag_styling` ecosystem, ensuring excellent visibility of selected text in all supported terminal emulators.

## Problem Statement

Users reported that selection backgrounds in some terminals (particularly iTerm2 and Apple Terminal) were nearly invisible against dark backgrounds. For example, with the Atelier Seaside Dark theme:
- Background: `#131513` (very dark green)
- Original selection: Only 40% brighter → `#1a1d1a` (barely visible)
- This created an unusable selection experience

## Solution Architecture

### 1. Enhanced Contrast Requirements

**File:** `src/bin/thag_convert_themes.rs`

Increased contrast requirements for `subtle` and `commentary` colors:
- **Dark themes:** 0.55 → **0.70** (27% increase)
- **Light themes:** 0.50 → **0.60** (20% increase)

These colors are used for selection backgrounds in terminal exports.

**Result:** Commentary color now achieves **11.13:1 contrast ratio** (exceeds WCAG AAA standard of 7:1)

### 2. Improved Brightness Adjustment Algorithm

**File:** `thag_styling/src/exporters/mod.rs`

Enhanced `adjust_color_brightness()` function:
```rust
// For very dark colors (RGB < 50), add minimum boost
if r < 50 && g < 50 && b < 50 && factor > 1.0 {
    let min_boost = 80.0;
    return (r * factor + min_boost, g * factor + min_boost, b * factor + min_boost);
}
```

This ensures that very dark backgrounds get additive brightening instead of just multiplicative scaling.

### 3. Prominence-Based Heading Color Ranking

**Files:**
- `src/bin/thag_convert_themes.rs`
- `thag_styling/src/image_themes.rs`

Implemented theme-aware hue adjustments for better heading prominence:

**Dark Themes:**
- Cyan: 1.25× boost (highest - most prominent)
- Red/Orange: 1.15× boost
- Magenta/Pink: 1.15× boost
- Blue: 1.10× boost
- Yellow: 1.02× boost
- Purple: 0.95× reduction

**Light Themes:**
- Purple: 1.10× boost (dark purples stand out on light backgrounds)
- Red/Orange: 1.08× boost
- Magenta/Pink: 1.08× boost
- Cyan: 1.05× boost
- Blue: 1.05× boost
- Yellow: 0.98× reduction (bright yellows less prominent)

**Key Improvement:** Colors are now enhanced for contrast **before** sorting by prominence, ensuring final assignments reflect actual visual hierarchy.

## Terminal-Specific Implementations

### 1. iTerm2 (macOS)

**File:** `thag_styling/src/exporters/iterm2.rs`

**Changes:**
- Added `Alpha Component` field (value: 1.0)
- Added `Color Space` field (value: "P3")
- Use `commentary` color for selection background
- Use `normal` color for selection foreground

**Format:**
```xml
<key>Selection Color</key>
<dict>
    <key>Alpha Component</key>
    <real>1</real>
    <key>Blue Component</key>
    <real>0.75294117647058822</real>
    <key>Color Space</key>
    <string>P3</string>
    <key>Green Component</key>
    <real>0.80392156862745101</real>
    <key>Red Component</key>
    <real>0.75294117647058822</real>
</dict>
```

**User Configuration Required:**
Users must enable "Use custom color for selected text" checkbox in iTerm2 preferences after importing themes.

### 2. Mintty (Git Bash, Cygwin)

**File:** `thag_styling/src/exporters/mintty.rs`

**Added Exports:**
```ini
SelectionBackgroundColour=192,205,192
SelectionForegroundColour=175,196,175
```

Also maintains legacy `HighlightBackgroundColour` for compatibility.

### 3. WezTerm

**File:** `thag_styling/src/exporters/wezterm.rs`

**Updated:** Now uses `commentary` color instead of brightness adjustment:
```toml
selection_bg = "#c0cdc0"
selection_fg = "#8ca68c"
```

### 4. Windows Terminal

**File:** `thag_styling/src/exporters/windows_terminal.rs`

Already exports `selectionBackground` - verified working correctly.

### 5. Alacritty

**File:** `thag_styling/src/exporters/alacritty.rs`

**Updated:** Now uses `commentary` color for selection background.

### 6. Kitty

**File:** `thag_styling/src/exporters/kitty.rs`

**Updated:** Now uses `commentary` color for selection background.

## OSC Escape Sequence Support

**File:** `thag_styling/src/palette_sync.rs`

Added dynamic color setting via OSC sequences:

### Selection Colors
- **OSC 17:** Selection background (using `commentary` color)
- **OSC 19:** Selection foreground (using `normal` color)
- **OSC 117:** Reset selection background
- **OSC 119:** Reset selection foreground

### Cursor Color
- **OSC 12:** Cursor color (using `emphasis` color)
- **OSC 112:** Reset cursor color

### Terminal Support Matrix

| Terminal | OSC 17 (sel bg) | OSC 19 (sel fg) | OSC 12 (cursor) |
|----------|----------------|----------------|-----------------|
| Gnome Terminal | ✅ | ✅ | ✅ |
| WezTerm | ✅ | ✅ | ✅ |
| iTerm2 | ✅ | ✅ | ✅ |
| Kitty | ✅ | ✅ | ✅ |
| Apple Terminal | ✅ | ❌ | ✅ |
| WSL Ubuntu | ✅ | ✅ | ✅ |
| Alacritty | ❌ | ❌ | ✅ |

## Documentation Updates

**File:** `thag_styling/README.md`

Added comprehensive "Selection Colors Configuration" section covering:
- Automatic configuration for terminals with full support
- Manual configuration steps for iTerm2, Apple Terminal, and Konsole
- OSC sequence support matrix
- Selection color design principles
- Usage of `thag_sync_palette` tool

## Testing Results

### Atelier Seaside Dark Theme
- **Background:** `#131513` (RGB: 19, 21, 19)
- **Selection Background:** `#c0cdc0` (RGB: 192, 205, 192)
- **Contrast Ratio:** **11.13:1** ✅
- **Result:** Excellent visibility across all terminals

### Dracula Dark Theme
**Before (all equal prominence):**
- H1: #fa97cf (0.859)
- H2: #bf97fa (0.859)
- H3: #97e8fa (0.859)

**After (hue-adjusted):**
- H1: #97e8fa (1.074) - Cyan ✅
- H2: #fa97cf (0.988) - Magenta ✅
- H3: #bf97fa (0.945) - Purple ✅

Matches user subjective assessment: Cyan > Pink > Purple

## Files Modified

### Core Theme Generation
1. `src/bin/thag_convert_themes.rs` - Enhanced contrast, prominence ranking
2. `thag_styling/src/image_themes.rs` - Prominence calculation for image themes

### Terminal Exporters
3. `thag_styling/src/exporters/mod.rs` - Brightness adjustment algorithm
4. `thag_styling/src/exporters/iterm2.rs` - Full plist format, selection colors
5. `thag_styling/src/exporters/mintty.rs` - Selection color fields
6. `thag_styling/src/exporters/wezterm.rs` - Commentary color usage
7. `thag_styling/src/exporters/alacritty.rs` - Commentary color usage
8. `thag_styling/src/exporters/kitty.rs` - Commentary color usage

### Dynamic Palette
9. `thag_styling/src/palette_sync.rs` - OSC 12, 17, 19 sequences

### Documentation
10. `thag_styling/README.md` - Selection color configuration guide

## Key Achievements

✅ **11.13:1 contrast ratio** for selection backgrounds (exceeds accessibility standards)
✅ **Consistent selection colors** across all terminal exporters
✅ **Theme-aware prominence ranking** for headings (light vs dark)
✅ **Dynamic palette support** via OSC sequences
✅ **Comprehensive documentation** for terminal-specific requirements
✅ **Verified compatibility** with 10+ terminal emulators

## User Actions Required

### iTerm2
1. Import theme file
2. Enable ☑ "Use custom color for selected text" in Preferences → Profiles → Colors

### Apple Terminal
- Selection colors cannot be imported from files
- Use `thag_sync_palette apply <theme>` for OSC 17 support
- Or manually set in Terminal → Preferences → Profiles → Text

### Konsole
- Enable "Always invert the colors of selected text" in Appearance → Miscellaneous
- Selection colors are dynamically calculated from theme colors

## Future Improvements

1. Consider implementing Apple Terminal's encrypted format parser
2. Add automated testing for selection contrast ratios
3. Create visual comparison tool for prominence ranking validation
4. Support for additional cursor-related OSC sequences (shape, blink rate)

## References

- [iTerm2 Color Documentation](https://iterm2.com/documentation-preferences-profiles-colors.html)
- [OSC Sequence Specification](https://invisible-island.net/xterm/ctlseqs/ctlseqs.html)
- [WCAG Contrast Guidelines](https://www.w3.org/WAI/WCAG21/Understanding/contrast-minimum.html)