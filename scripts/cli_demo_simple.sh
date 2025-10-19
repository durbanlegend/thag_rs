#!/bin/bash

# Simple CLI Demo Feeder for thag command-line demos
# Copies commands to clipboard - paste with Cmd-V and they won't auto-execute

lines=(
    "# Welcome to the thag CLI demo!"
    "thag -e '(1..=34).product::<u128>()'"
    "thag -e 'use jiff::{Zoned, Unit}; Zoned::now().round(Unit::Second)?'"
    "thag -e ' {
    use jiff::{Zoned, Unit};
    Zoned::now().round(Unit::Second)?
    }'"
    "thag -e '(1..=50).map(|x| x * x).filter(|x| x % 3 != 0).take(10).collect::<Vec<_>>()'"
    "echo -e 'hello\\nworld\\nrust' | thag --loop 'line.to_uppercase()'"
    "seq 1 10 | thag --begin 'let mut sum = 0;' --loop 'sum += line.parse::<i32>()?;' --end 'println!(\"Total: {}\", sum);'"
    "thag_show_themes"
    "thag -xe 'println!(\"{:.2}\", 235.215 / std::env::args().skip(1).next().expect(\"Expected a l/100km numeric value\").parse::<f64>()?);'  && mv ~/.cargo/bin/temp ~/.cargo/bin/to_mpg && echo Success"
)

index=0
max=$((${#lines[@]} - 1))

echo "========================================="
echo "CLI Demo Feeder"
echo "========================================="
echo ""
echo "Press Enter to copy next command to clipboard."
echo "Then paste in demo terminal with Cmd-V (won't auto-execute)."
echo "Press Enter in demo terminal to execute the command."
echo ""

read -p "Press Enter to start..." -r

while true; do
    if [ $index -le $max ]; then
        current="${lines[$index]}"

        echo ""
        echo "[$index/$max] $current"

        # Copy without trailing newline so it won't auto-execute
        echo -n "$current" | pbcopy

        echo "  ✓ Copied to clipboard"
        echo "  → Paste in demo terminal with Cmd-V, then press Enter to execute"

        ((index++))
    else
        echo ""
        echo "========================================="
        echo "Demo complete!"
        echo "========================================="
        break
    fi

    read -p "Press Enter for next command..." -r
done
