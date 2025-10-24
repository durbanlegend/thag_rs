# Demo Scripts for thag Presentations

This directory contains scripts to help with smooth live demonstrations of thag, eliminating the need to type complex commands during presentations.

## Scripts Overview

### 1. `demo_feeder.sh` - REPL Demo Helper

**Purpose:** For demonstrating the thag REPL with clipboard-based paste.

**How it works:**
- Displays each line of the demo script
- Copies it to clipboard
- You paste it into the iterator with Cmd-V
- Multi-line expressions wait for you to press Enter

**Usage:**
```bash
./scripts/demo_feeder.sh
# In REPL: Press Cmd-V to paste each line
```

**Best for:** REPL demonstrations where you're showing interactive features.

---

### 2. `demo_to_buffer.zsh` - CLI Demo Helper (RECOMMENDED)

**Purpose:** For command-line demos where you want commands to appear in your shell's editing buffer and history.

**How it works:**
- Uses zsh's `print -z` to push commands to your editing buffer
- Commands appear at your prompt as if you typed them
- Press Enter to execute
- Commands are saved in shell history (arrow up works!)
- No typing or pasting required

**Usage:**
```bash
# In your demo terminal (must be zsh):
source scripts/demo_to_buffer.zsh

# Then advance through demo:
demo_feed
# Press Enter to execute the command that appears
# Repeat demo_feed for next command

# Other commands:
demo_reset   # Start over
demo_stop    # Stop demo (skip to end)
demo_status  # Show current position
demo_list    # List all commands
```

**Pro tip:** Bind to a key for even smoother demos:
```bash
bindkey '^N' demo_feed_widget  # Press Ctrl-N to advance
```

**Best for:** Command-line demos where you want natural shell interaction and history.

**Limitations:**
- zsh only (won't work in bash)
- Must be sourced, not executed

---

### 3. `cli_demo_simple.sh` - Simple Clipboard Demo

**Purpose:** Simple clipboard-based approach for any shell.

**How it works:**
- Copies each command to clipboard (without newline)
- You paste with Cmd-V
- Command appears but doesn't execute
- Press Enter to execute

**Usage:**
```bash
./scripts/cli_demo_simple.sh
# When prompted, paste in demo terminal with Cmd-V
```

**Best for:**
- When you need bash compatibility
- Simple presentations without shell-specific features

**Limitations:**
- Single-line commands don't get added to history when pasted
- Arrow up recalls paste command, not the actual command

---

### 4. `cli_demo_typer.sh` - AppleScript-Based (Advanced)

**Purpose:** Automatically types commands into the active terminal using AppleScript.

**How it works:**
- Uses macOS AppleScript to simulate typing
- You switch to demo terminal, script types the command
- Command appears ready to execute

**Usage:**
```bash
./scripts/cli_demo_typer.sh
# Switch to demo terminal when prompted
```

**Best for:** Advanced users who want automated typing.

**Limitations:**
- macOS only
- Requires terminal app detection
- May have escaping issues with complex commands
- Timing can be tricky

---

## Which Script Should I Use?

### For REPL Demos:
→ Use `demo_feeder.sh`

### For Command-Line Demos:
→ **Recommended:** Use `demo_to_buffer.zsh` (if using zsh)
→ **Alternative:** Use `cli_demo_simple.sh` (if using bash or want simplicity)

## Customizing Demo Content

Edit the `lines=()` array in any script to customize your demo commands:

```bash
lines=(
    "# Your comment"
    "thag -e 'your_expression()'"
    "# Another comment"
    "thag --your-flag"
)
```

## Tips for Smooth Presentations

1. **Test first:** Run through your demo script before the actual presentation
2. **Clear your terminal:** `clear` before starting for a clean look
3. **Adjust font size:** Make sure your audience can read the text
4. **Use comments:** Add comment lines starting with `#` to explain what's happening
5. **Add pauses:** Blank lines in the array create natural breaks
6. **Practice timing:** Know when to talk vs. when to let the command speak

## Example Demo Workflow

```bash
# Terminal 1 (demo terminal):
source scripts/demo_to_buffer.zsh

# Now run your demo:
# Talk about thag...
demo_feed  # Command appears
# Explain what this command does...
# Press Enter to execute
# Show the output...
demo_feed  # Next command
# And so on...
```

## Troubleshooting

**Problem:** `print -z` doesn't work
- **Solution:** Make sure you're using zsh, not bash. Check with `echo $SHELL`

**Problem:** Commands execute immediately when pasted
- **Solution:** Use `demo_to_buffer.zsh` instead of clipboard paste for single-line commands

**Problem:** Arrow up doesn't recall pasted commands
- **Solution:** This is a shell limitation with paste. Use `demo_to_buffer.zsh` which uses `print -z` to add commands to history properly

**Problem:** Script can't find `pbcopy`
- **Solution:** These scripts are macOS-specific. On Linux, replace `pbcopy` with `xclip -selection clipboard` or similar

## Advanced: Creating Your Own Demo Script

1. Copy one of the existing scripts as a template
2. Edit the `lines=()` array with your commands
3. Save with a descriptive name
4. Make it executable: `chmod +x scripts/your_demo.sh`
5. Test it thoroughly!

---

**Questions or improvements?** Feel free to modify these scripts for your specific presentation needs!
