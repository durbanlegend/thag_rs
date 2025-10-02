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
- Choose a new color from all available palette colors
- Color previews show the actual color with `████` blocks

**Example**: Change Heading1 from purple to cyan

### 2. Swap two roles
- Select two roles to swap
- Colors are exchanged between them
- Useful for quickly fixing prominence issues

**Example**: Swap Heading1 and Heading3 if their prominence feels reversed

### 3. Reset to original
- Undo all changes made in this session
- Returns palette to the state it was when loaded
- Requires confirmation

### 4. Show current palette
- Display all 16 role colors with visual previews
- Shows role name, color block, and hex code
- Useful for reviewing your changes before saving

### 5. Save and exit
- Saves changes to the output file (or input file if no output specified)
- Creates backup file (`.toml.backup`) by default
- Prompts for confirmation if modified

### 6. Exit without saving
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
  ████ #cba6f7 (currently Heading1)
  ████ #f2cdcd (currently Heading2)
  ████ #eba0ac (currently Error)
  ❯ ████ #fab387 (currently Warning)
  ...

✅ Updated Heading3 to #fab387

# Select "Save and exit"
📦 Backup created: themes/catppuccin-mocha.toml.backup
💾 Theme saved: themes/catppuccin-mocha.toml
```

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

## Limitations

- Only works with `.toml` theme files (not `.yaml` base16/24 sources)
- Only edits colors, not attributes (bold, italic, etc.)
- Colors must already exist in the palette (can't add new colors)
- True color (RGB) themes only

## See Also

- `thag_convert_themes` - Convert base16/24 themes to thag format
- `thag_convert_themes_alt` - Convert with contrast enhancement
- `thag_sync_palette` - Apply themes to your terminal dynamically