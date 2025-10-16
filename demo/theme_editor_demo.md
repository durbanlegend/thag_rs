# Theme Editor Demo

This demo shows how to use `thag_edit_theme` to interactively customize theme colors.

## Purpose

The theme editor allows you to:
- **Edit individual color roles** - Change the color assigned to any role
- **Swap role colors** - Exchange colors between two roles
- **Preview changes** - See all colors with their current assignments
- **Reset changes** - Undo all modifications and return to original
- **Save/backup** - Save changes with automatic backup creation

## Basic Usage

```bash
# Edit a theme in place (creates backup automatically)
thag_edit_theme --input themes/my-theme.toml

# Edit and save to a different file
thag_edit_theme --input themes/original.toml --output themes/modified.toml

# Edit without creating backup
thag_edit_theme --input themes/my-theme.toml --no-backup
```

## Interactive Workflow

When you run the editor, you'll see a menu with these options:

### 1. Edit color role
- Select which role to edit (Heading1, Heading2, Error, etc.)
- See the current color with a visual preview
- Choose from ALL colors in the theme (including base_colors if available)
- Enhanced display shows:
  - Base16/24 indices (e.g., "base00, base01")
  - Current role assignments (e.g., "Subtle, Commentary")
  - Visual preview with `████` blocks

**Example**: Change Heading1 from purple to cyan

**New in v0.2**: If the theme has `base_colors`, you can select from the full base16/24 palette (16-24 colors) instead of just the 16 role colors. Each color shows which base indices and roles use it.

### 2. Adjust color
- Select a role to adjust
- Choose from preset adjustments:
  - Lighten (+10%)
  - Darken (-10%)
  - Increase saturation (+10%)
  - Decrease saturation (-10%)
- See before/after preview
- Confirm before applying

**Example**: Lighten Heading3 by 10% to improve readability

**Color Space**: Adjustments use HSL color space with conservative bounds:
- Lightness: clamped to 10-90% to maintain visibility
- Saturation: clamped to 0-100%
- Hue: preserved to stay in same color family

### 3. Swap two roles
- Select two roles to swap
- Colors are exchanged between them
- Useful for quickly fixing prominence issues

**Example**: Swap Heading1 and Heading3 if their prominence feels reversed

### 4. Reset to original
- Undo all changes made in this session
- Returns palette to the state it was when loaded
- Requires confirmation

### 5. Show current palette
- Display all 16 role colors with visual previews
- Shows role name, color block, and hex code
- Useful for reviewing your changes before saving

### 6. Save and exit
- Saves changes to the output file (or input file if no output specified)
- Creates backup file (`.toml.backup`) by default
- Prompts for confirmation if modified

### 7. Exit without saving
- Discards all changes
- Prompts for confirmation if modified

## Use Cases

### Case 1: Fix Automatic Conversion Issues

Sometimes automatic theme conversion doesn't match your preferences:

```bash
# Convert a base16/24 theme
thag_convert_themes_alt --input source-theme.yaml -o converted-theme.toml

# Open in editor to tweak heading colors
thag_edit_theme --input converted-theme.toml
```

Then interactively:
1. Choose "Show current palette" to see what was generated
2. Choose "Edit color role" to change Heading1, Heading2, or Heading3
3. Select from the available colors in the palette
4. Save when satisfied

### Case 2: Create Theme Variations

Start with a base theme and create variations:

```bash
# Create a new variant
cp themes/original.toml themes/variant1.toml

# Customize it interactively
thag_edit_theme --input themes/variant1.toml
```

### Case 3: Quick Color Swaps

If two colors just feel "wrong" in their roles:

```bash
thag_edit_theme --input themes/my-theme.toml
# Choose "Swap two roles"
# Select the two roles to swap
# Save
```

## Tips

1. **Use Show Palette First**: Before making changes, view the entire palette to understand what colors are available

2. **Backup is Your Friend**: The default backup creation means you can always revert

3. **Experiment Freely**: Use "Reset to original" to undo everything without saving

4. **Visual Feedback**: The colored `████` blocks show exactly what each color looks like in your terminal

5. **Color Reuse**: The editor shows which colors are currently assigned to which roles, helping you avoid unintended duplicates

## Example Session

```bash
$ thag_edit_theme --input themes/catppuccin-mocha.toml

🎨 Theme Editor: Catppuccin Mocha

📋 Theme: Catppuccin Mocha
🌓 Type: Dark
🎨 Color Support: TrueColor
🖼️  Background: #1e1e2e

? What would you like to do? 
  ❯ Edit color role
    Adjust color
    Swap two roles
    Reset to original
    Show current palette
    Save and exit
    Exit without saving

# Select "Show current palette"
📊 Current Palette:

  Heading1     │ ████ #cba6f7
  Heading2     │ ████ #f2cdcd
  Heading3     │ ████ #8cf2d5
  Error        │ ████ #f38ba8
  Warning      │ ████ #fab387
  ...

# Select "Edit color role"
? Which role would you like to edit? Heading3

Current color for Heading3: ████ #8cf2d5

? Select new color:
  ████ #1e1e2e (base00, base01 | Subtle, Commentary)
  ████ #cdd6f4 (base05 | Normal)
  ████ #f38ba8 (base08 | Error)
  ❯ ████ #fab387 (base09 | Warning)
  ████ #a6e3a1 (base0B | Link, Success)
  ████ #94e2d5 (base0C | Code, Heading3)
  ████ #cba6f7 (base0E | Heading1)
  ████ #f2cdcd (base0F | Heading2)
  ████ #eba0ac (base12)
  ████ #89dceb (base15)
  ...

✅ Updated Heading3 to #fab387

# Or use Adjust color:
? Which role would you like to adjust? Heading3

Current color for Heading3: ████ #94e2d5

? How would you like to adjust?
  Lighten (+10%)
  Darken (-10%)
  Increase saturation (+10%)
  ❯ Decrease saturation (-10%)
  Custom adjustment
  Cancel

Before: ████ #94e2d5
After:  ████ #8cd4d0

? Apply this adjustment? Yes

✅ Adjusted Heading3 to #8cd4d0

# Select "Save and exit"
📦 Backup created: themes/catppuccin-mocha.toml.backup
💾 Theme saved: themes/catppuccin-mocha.toml
```

## Enhanced Color Picker with base_colors

When a theme includes `base_colors` (generated by `thag_convert_themes`), the editor shows ALL colors from the original base16/24 palette:

### Display Format

```
████ #282a36 (base00, base01 | Subtle, Commentary)
```

This shows:
- **Visual preview**: `████` color block
- **Hex value**: `#282a36`
- **Base indices**: `base00, base01` - which base16/24 slots use this color
- **Roles**: `Subtle, Commentary` - which semantic roles use this color

### Benefits

1. **Full palette access**: Choose from 16-24 colors instead of just 16 role colors
2. **Avoid duplication**: See which colors are already used
3. **Theme coherence**: Stay within the original theme's color palette
4. **Color provenance**: Understand where each color comes from

## Color Adjustment Features

### Preset Adjustments

- **Lighten/Darken**: ±10% lightness in HSL space
- **Saturation**: ±10% saturation
- **Visual preview**: See before/after comparison
- **Reversible**: Can undo by resetting to original

### Color Space Constraints

To maintain theme coherence, adjustments use **conservative bounds**:

```
Lightness: 10% - 90%  (prevents pure black/white)
Saturation: 0% - 100% (full range allowed)
Hue: Preserved       (stays in same color family)
```

### Use Cases

1. **Improve contrast**: Lighten a too-dark color for better readability
2. **Tone down**: Desaturate an overly vibrant color
3. **Quick tweaks**: Small adjustments without leaving theme palette
4. **Accessibility**: Meet WCAG contrast requirements

## Integration with thag_sync_palette

After editing, you can apply your customized theme to your terminal:

```bash
# Edit the theme
thag_edit_theme --input ~/.config/thag/themes/my-theme.toml

# Apply it to your terminal
thag_sync_palette apply my-theme
```

## Files Created

- **Original file**: Updated with your changes (if saved)
- **Backup file**: `<filename>.toml.backup` (created by default)
- **No temporary files**: All changes are in memory until you save

## Advanced Features

### Working with base_colors

The editor automatically loads `base_colors` if present in the theme:

```bash
# Themes converted with the new converters include base_colors
thag_convert_themes_alt --input dracula.yaml -o themes/
thag_edit_theme --input themes/dracula.toml

# Full base16/24 palette available for selection!
```

### Color Sorting

Colors are displayed with intelligent sorting:
1. Colors with base indices (from original theme) shown first
2. Then sorted by number of role assignments
3. This puts the most "important" colors at the top

### Adjustment Workflow

```
1. Select "Adjust color"
2. Pick a role (e.g., Heading3)
3. Choose adjustment type
4. Preview the change
5. Confirm or cancel
```

## Limitations

- Only works with `.toml` theme files (not `.yaml` base16/24 sources)
- Only edits colors, not attributes (bold, italic, etc.)
- Color adjustments are HSL-based (may shift hue slightly in extreme cases)
- Custom adjustment values not yet implemented (use presets)
- True color (RGB) themes only

## See Also

- `thag_convert_themes` - Convert base16/24 themes to thag format (includes base_colors)
- `thag_convert_themes_alt` - Convert with contrast enhancement (includes base_colors)
- `thag_sync_palette` - Apply themes to your terminal dynamically (uses base_colors for ANSI mapping)
- `docs/base_colors_feature.md` - Technical documentation for base_colors feature