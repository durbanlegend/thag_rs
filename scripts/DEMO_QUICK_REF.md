# thag Demo Quick Reference Card

## Setup (One Time)

```bash
cd /path/to/thag_rs
source scripts/demo_to_buffer.zsh
```

**Optional:** Bind to Ctrl-N for faster demos:
```bash
bindkey '^N' demo_feed_widget
```

---

## Demo Commands

| Command | Purpose |
|---------|---------|
| `demo_feed` | Advance to next command (or press Ctrl-N if bound) |
| `demo_reset` | Restart from beginning |
| `demo_stop` | Stop demo (skip to end) |
| `demo_status` | Show current position |
| `demo_list` | List all demo commands |

---

## Demo Workflow

1. **Talk** - Explain what you'll demonstrate
2. **Advance** - Type `demo_feed` (or Ctrl-N)
3. **Explain** - Command appears, explain what it does
4. **Execute** - Press Enter to run
5. **Show** - Discuss the output
6. **Repeat** - Go to step 2

---

## Demo Script Commands

```bash
# 1. Simple factorial
thag -e '(1..=34).product::<u128>()'

# 2. Using external dependency (jiff)
thag -e 'use jiff::{Zoned, Unit}; Zoned::now().round(Unit::Second)?'

# 3. Multi-line expression
thag -e ' {
    use jiff::{Zoned, Unit};
    Zoned::now().round(Unit::Second)?
    }'

# 4. Complex iterator chain
thag -e '(1..=50).map(|x| x * x).filter(|x| x % 3 != 0).take(10).collect::<Vec<_>>()'

# 5. Loop over stdin
echo -e 'hello\nworld\nrust' | thag --loop 'line.to_uppercase()'

# 6. Begin/loop/end pattern
seq 1 10 | thag --begin 'let mut sum = 0;' --loop 'sum += line.parse::<i32>()?;' --end 'println!("Total: {}", sum);'

# 7. List themes
thag_show_themes

# 8. Generate a converter from litres per 100km to miles per gallon.
thag -xe 'println!("{:.2}", 235.215 / std::env::args().skip(1).next().expect("Expected a l/100km numeric value").parse::<f64>()?);'  && mv ~/.cargo/bin/temp ~/.cargo/bin/to_mpg && echo Success

# 9. View generated script
cat /tmp/sum_demo.rs

# 10. Help
thag --help
```

---

## Troubleshooting

| Problem | Solution |
|---------|----------|
| "No such widget" | Use `demo_feed_widget` not `demo_feed` for bindkey |
| Nothing happens | Make sure you **sourced** the script, not executed it |
| Commands execute immediately | This is correct! Press Enter to run them |
| Can't stop demo | Type `demo_stop` to skip to end |

---

## Tips for Great Demos

- ✓ Clear your terminal before starting
- ✓ Increase font size for visibility
- ✓ Explain BEFORE running each command
- ✓ Let the output speak - don't rush
- ✓ Practice the timing beforehand
- ✓ Have the quick ref open on another screen

---

**Pro Tip:** Print this file and keep it next to your laptop during presentations!
