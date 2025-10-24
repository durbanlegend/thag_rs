#!/bin/zsh

# Demo to Buffer - zsh function for feeding commands to the editing buffer
#
# Usage:
#   1. Source this file in your demo terminal: source demo_to_buffer.zsh
#   2. Run: demo_feed
#   3. Each command will appear in your prompt ready to execute
#   4. Press Enter to execute the command
#   5. Arrow up will recall it from history!
#
# This solves the problem where pasting single-line commands doesn't add them
# to shell history. Using print -z, commands appear in the editing buffer
# as if you typed them, so they go into history when executed.

# Array of demo commands
typeset -ga DEMO_LINES
DEMO_LINES=(
    ": # Welcome to the thag CLI demo!"
    ": #' Let's start by running a script. This one is a snippet that does a classic fast fibonacci calculation:"
    "cat demo/fib_classic_ibig.rs"
    ": #' Let's run it with thag to calculate the nth Fibonacci number. Using --force/-f to show rebuild speed:"
    "thag demo/fib_classic_ibig.rs -f -- 10000"
    ": #' Now let's check out the thag help:"
    "thag --help | less"
    ": # Now let's run some expressions. Here's factorial 34, which is as big as Rust integers can go:"
    "thag -e '(1..=34).product::<u128>()'"
    ": # We can go bigger with a big-number crate. Note no Cargo.toml info needed!"
    "thag -e 'use dashu::integer::UBig; println!(\"{}\", (1_u8..=100).map(UBig::from).product::<UBig>())'"
    ": # Note that these are precise calculations. Big factorials always end in many zeros because of all the multiples of 5 and 2 involved."
    ": # Multi-line expression with braces:"
    "thag -e ' {
    use jiff::{Zoned, Unit};
    Zoned::now().round(Unit::Second)?
    }'"
    ": #' let's use quiet mode -q to suppress compilation messages:"
    "thag -qe '(1..=50).map(|x| x * x).filter(|x| x % 3 != 0).take(10).collect::<Vec<_>>()'"
    ": # thag --loop/-l gives us a Unix-compliant filter:"
    "echo -e 'hello\\nworld\\nrust' | thag --loop 'line.to_uppercase()'"
    ": # ...and thag -qq gives any script or expression silent mode:"
    "seq 1 10 | thag --begin 'let mut sum = 0;' -qql 'sum += line.parse::<i32>()?;' --end 'println!(\"Total: {}\", sum);'"
    ": # So thag -qq allows us to pipe thag output to another process:"
    "echo -e \"foo\\\nbar\\\nbaz\" | thag -qqe 'use std::io::*; stdin().lines().map_while(Result::ok).for_each(|line| println!(r#\"{{\"name\": \"{}\"}}\"#, line));' | jq"
    ": #' thag accepts input from stdin. Let's first retrieve a gist I saved from the Rust Playground:"
    "curl -s https://gist.githubusercontent.com/rust-play/c95c26d75ca7eb43c42f0e17896a41ad/raw"
    ": #' Now let's pipe it to thag to run with the --stdin/-s option:"
    "curl -s https://gist.githubusercontent.com/rust-play/c95c26d75ca7eb43c42f0e17896a41ad/raw | thag -s"
    ": # thag can also edit and run the standard input with the --edit/-d option:"
    "curl -s https://gist.githubusercontent.com/rust-play/c95c26d75ca7eb43c42f0e17896a41ad/raw | thag -d"
    ": #' thag --edit/-d option can now recall this snippet from its history. Let's save it as a script and copy its location:"
    "thag -d"
    ": #' Now let's run the saved script:"
    "thag demo/my_gist.rs"
    ": # We also have a tool for remote scripts in thag_url:"
    "thag_url https://gist.github.com/rust-play/c95c26d75ca7eb43c42f0e17896a41ad/raw"
    ": # Any thag script or expression can be saved as a command by using the -x option to do a release build."
    ": #' Let's use an expression to create a tool that converts between litres per 100km and miles per US gallon."
    ": #' Fun fact: it's a reciprocal calculation that works both ways."
    "thag -xe 'println!(\"{:.2}\", 235.21 / std::env::args().skip(1).next().expect(\"Expected a l/100km numeric value\").parse::<f64>()?);'  && mv ~/.cargo/bin/temp ~/.cargo/bin/fueleconv && echo Success"
    ": #' Now let's run the converter:"
    "fueleconv 6.4"
    "fueleconv 30"
    ": #' Finally let's have a quick look at the iterator:"
    "thag -r"
    ": # The iterator is covered more fully in a separate demo"
    ": # Thanks for watching!"
)

# Global index for tracking position (1-based indexing for zsh arrays)
typeset -gi DEMO_INDEX=1

# Debug mode (set to 1 to see what's happening)
typeset -gi DEMO_DEBUG=0

# Function to feed next command to the editing buffer
demo_feed() {
    local max=${#DEMO_LINES[@]}

    if (( DEMO_INDEX > max )); then
        echo "Demo complete! All commands shown."
        echo "To restart: demo_reset"
        return 1
    fi

    local current="${DEMO_LINES[$DEMO_INDEX]}"

    if (( DEMO_DEBUG )); then
        echo "[DEBUG] Feeding line $DEMO_INDEX of $max"
    fi

    # Push to editing buffer - will appear at your prompt!
    print -z "$current"

    # Increment index AFTER feeding
    DEMO_INDEX=$(( DEMO_INDEX + 1 ))

    if (( DEMO_DEBUG )); then
        echo "[DEBUG] Next index will be: $DEMO_INDEX"
    fi
}

# ZLE widget wrapper for keybinding
demo_feed_widget() {
    demo_feed
    zle reset-prompt
}

# Register as a ZLE widget
zle -N demo_feed_widget

# Function to reset demo
demo_reset() {
    DEMO_INDEX=1
    echo "Demo reset to beginning."
    if (( DEMO_DEBUG )); then
        echo "[DEBUG] DEMO_INDEX set to: $DEMO_INDEX"
    fi
    demo_status
}

# Function to stop demo (skip to end)
demo_stop() {
    local max=${#DEMO_LINES[@]}
    DEMO_INDEX=$((max + 1))
    echo "Demo stopped. To restart: demo_reset"
}

# Function to show current demo status
demo_status() {
    local max=${#DEMO_LINES[@]}
    if (( DEMO_INDEX <= max )); then
        echo "Demo status: $DEMO_INDEX / $max"
        echo "Next: ${DEMO_LINES[$DEMO_INDEX]}"
    else
        echo "Demo complete! ($max / $max)"
        echo "To restart: demo_reset"
    fi
}

# Function to list all demo commands
demo_list() {
    local max=${#DEMO_LINES[@]}
    echo "Demo commands ($max total):"
    for i in {1..$max}; do
        echo "  [$i] ${DEMO_LINES[$i]}"
    done
}

# Function to enable debug mode
demo_debug() {
    if (( DEMO_DEBUG )); then
        DEMO_DEBUG=0
        echo "Debug mode OFF"
    else
        DEMO_DEBUG=1
        echo "Debug mode ON - you'll see index tracking info"
    fi
}

echo ""
echo "========================================="
echo "Demo Feeder Loaded!"
echo "========================================="
echo ""
echo "Available commands:"
echo "  demo_feed    - Feed next command to your prompt"
echo "  demo_reset   - Reset to beginning"
echo "  demo_stop    - Stop demo (skip to end)"
echo "  demo_status  - Show current position"
echo "  demo_list    - List all commands"
echo "  demo_debug   - Toggle debug mode"
echo ""
echo "Quick start:"
echo "  1. Type: demo_feed"
echo "  2. Press Enter to execute the command"
echo "  3. Repeat!"
echo ""
echo "Tip: Bind to a key for smooth demos!"
echo "  bindkey '^N' demo_feed_widget  # Ctrl-N to advance"
echo ""
