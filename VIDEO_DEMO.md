# Video Demo Guide for thag_rs v0.2.0

This document describes the recommended demo workflow for showcasing thag's TUI editor capabilities in the v0.2.0 release.

## Demo Overview

**Two-Session Structure**: Progressive demonstration of thag's TUI capabilities

### Session 1: Edit & Run Workflow
**Goal**: Demonstrate the complete workflow of editing and running a Rust script in thag's TUI editor.

**Duration**: ~45-60 seconds

**Key Features Demonstrated**:
- Loading existing scripts into the editor via stdin
- Making quick edits
- Discovering key bindings (Ctrl-L)
- Saving files with the file dialog
- Running and compiling code

### Session 2: Data Composition & Management
**Goal**: Show advanced data handling - combining sources, buffer operations, clipboard integration

**Duration**: ~60-90 seconds

**Key Features Demonstrated**:
- Copying data to clipboard with `thag_copy`
- Opening TUI editor empty
- Retrieving scripts from edit history
- Pasting clipboard contents
- Internal TextArea buffer operations (Ctrl-X yank, Ctrl-Y paste)
- System clipboard integration (F9 copy, F10 restore)
- Saving without execution
- Verifying clipboard contents with `thag_copy`

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

---

## Session 2: Step-by-Step Demo Script

### 1. Copy Data to Clipboard (0:00-0:05)
**Command**: `echo "Some data to process" | thag_copy`

**What happens**: Text is copied to system clipboard

**Alternative**: Copy any text from another source (browser, file, etc.)

### 2. Open Empty TUI Editor (0:05-0:08)
**Command**: `thag -d`

**What happens**: TUI editor opens with empty buffer

### 3. Retrieve Script from History (0:08-0:15)
**Action**: Navigate edit history (Up/Down arrows or history keys)

**What happens**: Previous script appears in edit buffer (e.g., a data processing template)

**Example script that might be in history**:
```rust
use std::io::{self, BufRead};

fn main() {
    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let line = line.unwrap();
        // Process data here
        println!("{}", line);
    }
}
```

### 4. Paste Clipboard Contents (0:15-0:20)
**Action**: Position cursor and paste with `Cmd-V` (or `Ctrl-Shift-V`)

**What happens**: Clipboard text appears in editor

### 5. Move Text with TextArea Buffer (0:20-0:35)
**Actions**:
- Select text to move (or position cursor on line)
- Press `Ctrl-X` to yank (cut) text into TextArea buffer
- Move cursor to new location
- Press `Ctrl-Y` to paste from TextArea buffer

**What happens**: Text moves within the document using thag's internal buffer

**Note**: This is separate from system clipboard - useful for quick rearrangements

### 6. Copy to System Clipboard (0:35-0:40)
**Actions**:
- Select or position on text you want to copy
- Press `F9` to copy current line/selection to system clipboard
- *Visual feedback in status bar*

### 7. Toggle Line Numbers (0:40-0:42)
**Action**: Press `F10` to restore normal view

**What happens**: Shows the F9/F10 toggle capability

**Note**: F9 also disables mouse capture for native terminal selection

### 8. Save Without Running (0:42-0:55)
**Action**: Press `Ctrl-S`

**Steps**:
- Navigate to desired directory
- Enter filename: `data_processor.rs`
- Press Enter

**What happens**: File saved, returns to editor

### 9. Exit Editor (0:55-0:58)
**Action**: Press `Ctrl-C` or `Ctrl-Q`

**What happens**: Exit without submitting for execution

### 10. Verify Clipboard Contents (0:58-1:05)
**Command**: `thag_copy`

**What happens**: Displays the text we copied with F9, confirming clipboard integration worked

**Result**: Shows the line/text we copied in step 6

---

## Combined Narrative

**Session 1** demonstrates the basic edit-compile-run cycle - the "hello world" of thag's TUI. It's quick, clean, and shows the complete workflow from input to output.

**Session 2** shows real-world data composition - combining history, clipboard data, and internal buffer operations. This demonstrates how thag can be part of a larger workflow, integrating with other tools and your clipboard to compose solutions from multiple sources. The save-without-run feature shows thag as a script editor, not just a runner.

Together, they present thag as both a rapid execution environment AND a capable development tool.

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

### Session 2 Captions

```
[0:00-0:05]
Copy data to clipboard
→ echo "Some data" | thag_copy

[0:05-0:08]
Open empty TUI editor
→ thag -d

[0:08-0:15]
Retrieve previous script from history
→ Navigate with arrow keys

[0:15-0:20]
Paste clipboard contents
→ Cmd-V

[0:20-0:35]
Move text with TextArea buffer
→ Ctrl-X to yank (cut)
→ Ctrl-Y to paste

[0:35-0:40]
Copy to system clipboard
→ F9 (copies current line)

[0:40-0:42]
Toggle display mode
→ F10

[0:42-0:55]
Save file without running
→ Ctrl-S
→ Filename: data_processor.rs

[0:55-0:58]
Exit editor
→ Ctrl-C

[0:58-1:05]
Verify clipboard contents
→ thag_copy
✓ Shows copied text
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

**Session 1**: Watch as we load an existing script, make quick edits, and run it—all from within thag's built-in terminal editor. The demo shows key discovery (Ctrl-L), the file save dialog, and the complete edit-compile-run cycle in under a minute.

**Session 2**: See advanced data composition in action - combining clipboard contents with scripts from history, using internal buffer operations to rearrange code, and integrating with the system clipboard. This session showcases thag's flexibility as a script composition tool.

### Long Version (for Release Notes / Blog)

**Interactive TUI Editor Workflow: A Two-Part Journey**

thag v0.2.0 includes a powerful TUI (Terminal User Interface) editor that provides a complete development environment right in your terminal. This two-session demo showcases both basic and advanced workflows:

#### Session 1: Edit & Run - The Foundation

Starting with an existing FizzBuzz script (`demo/fizz_buzz_gpt.rs`), we load it directly into the editor using stdin redirection. The code is immediately ready to edit—no project setup, no file creation, just start typing.

After making quick modifications (removing doc comments and adjusting the range for compact output), we demonstrate the discoverable key bindings system with Ctrl-L. This shows users how to find help without leaving the editor—a thoughtful UX detail that makes the tool approachable.

The file save dialog displays thag's integrated file browser, allowing navigation through directories and saving with a custom filename. The status bar confirms the save location, maintaining clear feedback throughout the workflow.

Finally, pressing Ctrl-D submits the code for compilation and execution. thag handles all the build machinery behind the scenes—dependency inference, compilation with cargo, and execution—presenting just the output that matters: a clean FizzBuzz sequence from 1 to 16.

This complete cycle—from loading a script to seeing results—takes less than a minute and demonstrates how thag removes friction from the Rust development experience while maintaining the power and safety of the full compiler toolchain.

#### Session 2: Data Composition - The Power User's Toolkit

The second session elevates thag from a simple script runner to a sophisticated composition tool. We start by copying data to the clipboard using `thag_copy`, then open an empty editor with `thag -d`.

Instead of starting from scratch, we retrieve a previous script from thag's edit history—perhaps a data processing template we've used before. This history feature makes thag feel less like a one-shot executor and more like a development environment that remembers your context.

We paste our clipboard contents directly into the editor, demonstrating integration with the system clipboard. But thag also has its own internal TextArea buffer, accessed via Ctrl-X (yank) and Ctrl-Y (paste). This dual-clipboard model—one for system integration, one for fast internal operations—gives users flexibility without complexity.

The F9 key captures the current line to the system clipboard, perfect for extracting snippets or sharing code. F10 toggles line numbers and mouse capture, showing thoughtful attention to different use cases (editing vs. selecting text for external copy).

Rather than running the code, we save it with Ctrl-S and exit with Ctrl-C, demonstrating that thag can serve as a script editor, not just a runner. Finally, we verify the clipboard operation with `thag_copy`, confirming the round-trip workflow.

This session showcases thag as part of a larger ecosystem—a tool that plays well with your clipboard, your history, and your existing workflow. It's not trying to replace your editor; it's augmenting your terminal toolkit with Rust superpowers.

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
