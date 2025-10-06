# Video Demo Guide for thag_rs v0.2.0

This document describes the recommended demo workflow for showcasing thag's TUI editor capabilities in the v0.2.0 release.

## Demo Overview

**Goal**: Demonstrate the complete workflow of editing and running a Rust script in thag's TUI editor.

**Duration**: ~45-60 seconds

**Key Features Demonstrated**:
- Invoke the the editor with optional stdin input from demo/fizz_buzz_gpt.rs. Not shown: clipboard paste, history retrieval
- Making quick edits
- Discovering key bindings (Ctrl-l)
- Saving files with the file dialog
- Running and compiling code

## Step-by-Step Demo Script

### 1. Load Script (0:00-0:04)
**Command**: `thag -d < demo/fizz_buzz_gpt.rs`

**What happens**: The TUI editor opens with the fizz_buzz_gpt.rs script loaded

### 2. Edit the Script (0:04-0:18)
**Actions**:
- Delete the first 3 lines (doc comment lines):
  ```
  //: GPT-generated fizz-buzz example.
  //# Purpose: Demo running random snippets in thag_rs, also AI and the art of delegation ;)
  //# Categories: learning, technique
  ```
- Change `1..=100` to `1..=16` (to show compact, complete output)

**Result**: Clean, minimal fizz-buzz that will display nicely

### 3. Show Key Mappings (0:18-0:26)
**Action**: Press `Ctrl-l`

**What happens**: Scrollable help panel appears showing all available key bindings

**Pause**: Hold for 2-3 seconds so viewers can see the bindings

### 4. Save File (0:26-0:50)
**Action**: Press `Ctrl-s or F12`

**What happens**: File save dialog appears

**Steps**:
- Navigate down to `demo/` directory
- Tab to filename field
- Type: `fizz_buzz_demo.rs`
- Press Enter

**Result**: Returns to editor with status message showing save location

### 5. Submit and Run (0:50-0:53)
**Action**: Press `Ctrl-d`

**What happens**: Editor submits the code for compilation

### 6. Watch Result (0:53-0:56)
**What happens**:
- Compilation progress
- Execution output showing:
  ```
  1
  2
  Fizz
  4
  Buzz
  Fizz
  7
  8
  Fizz
  Buzz
  11
  Fizz
  13
  14
  FizzBuzz
  16
  ```

**End**: Fade out or cut after output is visible

## Video Caption Text

For use as overlays or subtitles in the final video:

```
[0:00-0:04]
Load existing script into TUI editor
→ thag -d < demo/fizz_buzz_gpt.rs

[0:04-0:18]
Edit the script
• Remove doc comment lines
• Change range: 1..=100 → 1..=16

[0:18-0:26]
View available key bindings
→ Ctrl-l

[0:26-0:50]
Save to new file
→ Ctrl-s or F12
→ Navigate to demo/
→ Filename: fizz_buzz_demo.rs

[0:50-0:53]
Submit and run
→ Ctrl-d

[0:53-0:56]
Compilation and execution
✓ FizzBuzz output for 1-16
```

## Recording Options

### Option 1: Asciinema + Video Editor (Recommended)
1. Record with: `asciinema rec demo.cast`
2. Convert to video: `agg demo.cast demo.gif` (or use asciicast2gif)
3. Import into iMovie/Final Cut/QuickTime
4. Add text overlays using video editor
5. Export as MP4

**Pros**: Clean terminal recording, professional overlays
**Cons**: Requires conversion and video editing

### Option 2: QuickTime Screen Recording (Simplest)
1. Open QuickTime Player → New Screen Recording
2. Record terminal window directly
3. Add overlays in QuickTime or iMovie

**Pros**: Native Mac workflow, no conversion needed
**Cons**: Captures entire screen (may need cropping)

### Option 3: Asciinema with Typed Commentary
1. Record with asciinema
2. Type explanatory text into terminal before each step
3. Example: `echo "Step 1: Load script"` before running command

**Pros**: Self-contained, no post-processing
**Cons**: Less polished, interrupts flow

## Script Description for Documentation

### Short Version (for README)

**TUI Editor Demo**: Watch as we load an existing script, make quick edits, save it to a new file, and run it—all from within thag's built-in terminal editor. The demo shows key discovery (Ctrl-l), the file save dialog, and the complete edit-compile-run cycle in under a minute.

### Long Version (for Release Notes / Blog)

**Interactive TUI Editor Workflow**

thag v0.2.0 includes a powerful TUI (Terminal User Interface) editor that provides a complete development environment right in your terminal. This demo showcases the core workflow:

Starting with an existing FizzBuzz script (`demo/fizz_buzz_gpt.rs`), we load it directly into the editor using stdin redirection. The code is immediately ready to edit—no project setup, no file creation, just start typing.

After making quick modifications (removing doc comments and adjusting the range for compact output), we demonstrate the discoverable key bindings system with Ctrl-l. This shows users how to find help without leaving the editor—a thoughtful UX detail that makes the tool approachable.

The file save dialog displays thag's integrated file browser, allowing navigation through directories and saving with a custom filename. The status bar confirms the save location, maintaining clear feedback throughout the workflow.

Finally, pressing Ctrl-d submits the code for compilation and execution. thag handles all the build machinery behind the scenes—dependency inference, compilation with cargo, and execution—presenting just the output that matters: a clean FizzBuzz sequence from 1 to 16.

This complete cycle—from loading a script to seeing results—takes less than a minute and demonstrates how thag removes friction from the Rust development experience while maintaining the power and safety of the full compiler toolchain.

## Technical Notes

**Terminal Setup**:
- Use a theme with good contrast (consider using a thag_styling theme!)
- Font size: 14-16pt (ensure readability in video)
- Terminal dimensions: 100x30 or larger
- Consider setting `THAG_VERBOSITY=normal` for clean output

**Recording Settings**:
- If using asciinema: `asciinema rec --idle-time-limit 2.0` (caps pauses at 2s)
- Frame rate: 30fps minimum for smooth playback
- Resolution: 1920x1080 or higher for HD quality

**Environment**:
```bash
# Ensure clean state
cd ~/projects/thag_rs
export THAG_DEV_PATH=$PWD  # If using thag-auto dependencies
cargo build --release      # Ensure latest build

# Consider setting theme
export THAG_THEME=your-preferred-theme
```

## Embed Instructions

### For GitHub README

```markdown
### Video Demo

Watch thag's TUI editor in action:

[![thag TUI Editor Demo](thumbnail.png)](demo_video.mp4)

Or view the [full demo on YouTube/Vimeo](link).
```

### For Website/Blog

```html
<video width="100%" controls>
  <source src="thag_tui_demo.mp4" type="video/mp4">
  Your browser does not support the video tag.
</video>
```

### For Asciinema Player

```html
<asciinema-player src="demo.cast" cols="100" rows="30"></asciinema-player>
```

## Follow-up Video Ideas

Consider these additional demos for future releases:

- **REPL Mode**: Interactive Rust experimentation
- **URL Loading**: Running code directly from GitHub URLs with `thag_url`
- **Dependency Inference**: How thag automatically detects crates
- **Theme System**: Showcasing thag_styling themes
- **Command Building**: Converting scripts to fast compiled binaries
- **Tips & Tricks**: F9/F10 toggles, advanced editing, keyboard shortcuts

---

**Questions or suggestions?** Open an issue or PR!
