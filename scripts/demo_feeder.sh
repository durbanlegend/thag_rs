#!/bin/bash
lines=(
    "# Here's a short demo of the thag REPL."
    "# Evaluating a single-line expression:"
    "# We can get as gnarly as we like. Let's compute a series of cubes:"
    '(1..=50).map(|x| x * x).filter(|x| x % 3 != 0).enumerate().map(|(i, v)| format!("{}:{}", i, v)).collect::<Vec<_>>().join(", ")'
    "# Evaluating a multi-line expression:"
    "{
let nums = vec![1, 2, 3, 4];
nums.iter().sum::<i32>()
}"
    "# Using the built-in TUI Editor to convert the sum into a product, and Ctrl-d to re-submit and return:"
    "# You can also save the expression to a 'thag' script file from the TUI Editor."
    "# Alternatively we have an Edit command to invoke your favourite Editor:  VS Code, nano or what have you."
    "# I'll invoke Zed to overwrite the expression with '(1..=30).product::<u128>()'"
    "(1..=30).product::<u128>()"
    "# Now to re-evaluate it:"
    "# We can also go and tidy up the History:"
    "# There's built in Help, Keyboard mappings, a display of the current Theme and terminal attributes, and more:"
    "# Thanks for watching!"
)

index=0
max=$((${#lines[@]} - 1))

echo "Demo Feeder Ready! Press Enter to advance to next line."
echo "Current line will be copied to clipboard."
echo ""

while true; do
    if [ $index -le $max ]; then
        echo "[$index/$max] ${lines[$index]}"
        echo -n "${lines[$index]}" | pbcopy
        echo "  → Copied to clipboard. Press Cmd-V in terminal."
        ((index++))
    else
        echo "Demo complete!"
        break
    fi

    # Wait for Ctrl-C to continue
    trap 'echo ""' INT
    read -r
done
