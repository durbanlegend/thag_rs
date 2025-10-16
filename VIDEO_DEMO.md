# Video Demo Guide for thag_rs v0.2.0

> **Note**: This is internal planning documentation for recording demo videos.
> The completed demos are embedded in [README.md](README.md).

This document describes the recommended demo workflow for showcasing thag's TUI editor capabilities in the v0.2.0 release.

## Demo Overview

**Two-Session Structure**: Progressive demonstration of thag's TUI capabilities

**Recorded Demos**:
- Session 1: https://asciinema.org/a/nB3lFb6LgaHOF1s3dm5srjwyY
- Session 2: https://asciinema.org/a/LvSHLiZPC6lfCgSN4Q0sUJjpG
- REPL Demo: (existing recording)

### Session 1: Edit & Run Workflow
**Goal**: Use thag TUI editor to intercept, edit and run Rust code from stdin.

**Duration**: 30 seconds

**Key Features Demonstrated**:
- Loading existing scripts into the editor via stdin
- Making quick edits
- Discovering key bindings (Ctrl-L)
- Submitting and running code (Ctrl-D)

### Session 2: Data Composition & Management
**Goal**: Copy a range of lines from an existing script with thag_copy tool, compose with history in TUI editor, and run.

**Duration**: 1 minute 14 seconds

**Key Features Demonstrated**:
- Copying lines from a script with `thag_copy`
- Verifying clipboard contents with `thag_paste`
- Opening TUI editor and retrieving skeleton from history
- Pasting clipboard contents (Cmd-V/Ctrl-V)
- Internal TextArea buffer operations (Ctrl-X yank, Ctrl-Y paste) to rearrange code
- Submitting with Ctrl-D
- Compilation and execution

## Session 1: Step-by-Step Demo Script

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

### 4. Show Key Mappings (0:18-0:23)
**Action**: Press `Ctrl-L`

**What happens**: Scrollable help panel appears showing all available key bindings

**Pause**: Brief pause to show the bindings

### 5. Submit and Run (0:23-0:27)
**Action**: Press `Ctrl-d`

**What happens**: Editor submits the code for compilation

### 6. Watch Result (0:27-0:30)
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

---

## Session 2: Step-by-Step Demo Script

### 1. Copy Lines from Script (0:00-0:08)
**Command**: Extract lines from an existing script and copy to clipboard

**Example**: `sed -n '10,20p' demo/some_script.rs | thag_copy`

**What happens**: Lines are copied to system clipboard

### 2. Verify Clipboard (0:08-0:12)
**Command**: `thag_paste`

**What happens**: Displays the copied content to verify it's correct

### 3. Open TUI Editor (0:12-0:15)
</parameter>
**Command**: `thag -d`

**What happens**: TUI editor opens with empty buffer

### 4. Retrieve Script Skeleton from History (0:15-0:25)
**Action**: Navigate edit history to retrieve a previously saved skeleton

**What happens**: A script template appears in edit buffer

### 5. Paste Clipboard Contents (0:25-0:30)
**Action**: Position cursor and paste with `Cmd-V` (or `Ctrl-V`)

**What happens**: Clipboard text appears in editor

### 6. Move Import Line with TextArea Buffer (0:30-0:50)
**Actions**:
- Position cursor on import line that needs to move
- Press `Ctrl-X` to yank (cut) the line into TextArea buffer
- Move cursor to the top of the file
- Press `Ctrl-Y` to paste from TextArea buffer

**What happens**: Import line moves to the top using thag's internal buffer

**Note**: This is separate from system clipboard - useful for quick code rearrangements

### 7. Submit and Run (0:50-0:58)
**Action**: Press `Ctrl-D`

**What happens**: Code is submitted for compilation and execution

### 8. Watch Compilation and Results (0:58-1:14)
**What happens**:
- Compilation progress displayed
- Program executes
- Output shown in terminal

---

## Combined Narrative

**Session 1** (30 seconds) demonstrates the essential edit-and-run workflow. Load code from stdin, make quick edits, check available key bindings, and submit for execution. It's the "hello world" of thag's TUI - fast and straightforward.

**Session 2** (1:14) shows real-world code composition. Copy lines from an existing script with `thag_copy`, verify with `thag_paste`, retrieve a skeleton from edit history, paste the clipboard contents, and use the internal TextArea buffer (Ctrl-X/Ctrl-Y) to rearrange code before submitting. This demonstrates how thag integrates with your clipboard and command-line workflow to compose solutions from multiple sources.

Together, they present thag as both a rapid execution environment (Session 1) and a flexible composition tool (Session 2).

## Video Caption Text

### Session 1 Captions

**Title**: Thag TUI demo 1

**Description**: Use thag TUI editor to intercept, edit and run Rust code from stdin.

**URL**: https://asciinema.org/a/nB3lFb6LgaHOF1s3dm5srjwyY

**Markers**: None needed (simple 30-second flow)
```
[0:00-0:04]
Load existing script into TUI editor
→ thag -d < demo/fizz_buzz_gpt.rs

[0:04-0:18]
Edit the script
• Remove doc comment lines
• Change range: 1..=100 → 1..=16

[0:18-0:23]
View available key bindings
→ Ctrl-L

[0:23-0:27]
Submit and run
→ Ctrl-D

[0:27-0:30]
Compilation and execution
✓ FizzBuzz output for 1-16
```

### Session 2 Captions

**Title**: Thag TUI demo 2

**Description**: Copy a range of lines from an existing script with thag_copy tool. Verify with thag_paste tool. Open thag TUI editor and retrieve skeleton saved to history. Paste in lines from system clipboard (Ctrl-v or Cmd-v for Mac). Use editor buffer yank (Ctrl-x) and paste (Ctrl-y ) to move import up to the top. Use Ctrl-d to submit. Watch it compile and see results.

**URL**: https://asciinema.org/a/LvSHLiZPC6lfCgSN4Q0sUJjpG

**Markers**: None needed (straightforward workflow)

```
[0:00-0:08]
Copy lines from existing script
→ Extract and pipe to thag_copy

[0:08-0:12]
Verify clipboard contents
→ thag_paste

[0:12-0:15]
Open TUI editor
→ thag -d

[0:15-0:25]
Retrieve script skeleton from history
→ Navigate edit history

[0:25-0:30]
Paste clipboard contents
→ Cmd-V (Mac) or Ctrl-V

[0:30-0:50]
Move import with TextArea buffer
→ Ctrl-X to yank (cut)
→ Ctrl-Y to paste at top

[0:50-0:58]
Submit for execution
→ Ctrl-D

[0:58-1:14]
Compilation and results
✓ Code compiles and runs
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

**TUI Editor Demo (2 Sessions)**:

**Session 1** (30s): Watch as we load an existing script, make quick edits, and run it—all from within thag's built-in terminal editor. The demo shows key discovery (Ctrl-L) and the complete edit-compile-run cycle. https://asciinema.org/a/nB3lFb6LgaHOF1s3dm5srjwyY

**Session 2** (1:14): See advanced data composition in action - copying lines from an existing script with `thag_copy`, verifying with `thag_paste`, retrieving a skeleton from history, pasting clipboard contents, and using internal buffer operations (Ctrl-X/Ctrl-Y) to rearrange code before submitting. https://asciinema.org/a/LvSHLiZPC6lfCgSN4Q0sUJjpG

### Long Version (for Release Notes / Blog)

**Interactive TUI Editor Workflow: A Two-Part Journey**

thag v0.2.0 includes a powerful TUI (Terminal User Interface) editor that provides a complete development environment right in your terminal. This two-session demo showcases both basic and advanced workflows:

#### Session 1: Edit & Run - The Foundation (30 seconds)

Starting with an existing FizzBuzz script (`demo/fizz_buzz_gpt.rs`), we load it directly into the editor using stdin redirection. The code is immediately ready to edit—no project setup, no file creation, just start typing.

After making quick modifications (removing doc comments and adjusting the range for compact output), we demonstrate the discoverable key bindings system with Ctrl-L. This shows users how to find help without leaving the editor—a thoughtful UX detail that makes the tool approachable.

Pressing Ctrl-D submits the code for compilation and execution. thag handles all the build machinery behind the scenes—dependency inference, compilation with cargo, and execution—presenting just the output that matters: a clean FizzBuzz sequence from 1 to 16.

This complete cycle—from loading a script to seeing results—takes just 30 seconds and demonstrates how thag removes friction from the Rust development experience while maintaining the power and safety of the full compiler toolchain.

**Watch**: https://asciinema.org/a/nB3lFb6LgaHOF1s3dm5srjwyY

#### Session 2: Data Composition - The Power User's Toolkit (1:14)

The second session elevates thag from a simple script runner to a sophisticated composition tool. We start by copying a range of lines from an existing script using command-line tools piped to `thag_copy`. We verify the clipboard contents with `thag_paste` before opening the editor.

Opening the TUI editor with `thag -d`, we retrieve a previously saved skeleton from thag's edit history. This history feature makes thag feel less like a one-shot executor and more like a development environment that remembers your context.

We paste our clipboard contents directly into the editor with Cmd-V (or Ctrl-V), demonstrating integration with the system clipboard. But thag also has its own internal TextArea buffer, accessed via Ctrl-X (yank) and Ctrl-Y (paste). We use this to move an import statement to the top of the file—a common code organization task.

This dual-clipboard model—one for system integration, one for fast internal operations—gives users flexibility without complexity. After rearranging the code, we submit with Ctrl-D and watch it compile and execute.

This session showcases thag as part of a larger ecosystem—a tool that plays well with your clipboard, your history, and your existing workflow. It's not trying to replace your editor; it's augmenting your terminal toolkit with Rust superpowers.

**Watch**: https://asciinema.org/a/LvSHLiZPC6lfCgSN4Q0sUJjpG

**Together**, these sessions present thag's TUI as both approachable for beginners (Session 1) and powerful for experienced users (Session 2)—a rare combination that makes Rust experimentation faster and more enjoyable for everyone.

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
- **Loop-Filter Mode**: Data processing pipelines with `-l` flag
- **Multi-file Projects**: Working with modules and multiple source files
- **Debugging**: Using `thag_debug` and other analysis tools

---

**Questions or suggestions?** Open an issue or PR!
