


Wezterm provides a feature to change theme from the config file `~/.wezterm.lua`:

config.color_scheme = 'summercamp (base16)'

This normally works well and takes effect instantaneously, but the example above, with the name given by Wezterm's website (https://wezfurlong.org/wezterm/colorschemes/s/index.html#summercamp-base16) doesn't work, and I was wondering how to display a list of the supported themes in case the problem is just a typo on the website. I'm not a lua user and don't know how to run Wezterm lua scripts. Can you help?

Yes! You can list all available color schemes in Wezterm using Lua code in your config file. Here's how you can do it:

Add this to your `~/.wezterm.lua`:

```lua
local wezterm = require 'wezterm'

local config = {}

-- This will print all available color schemes to the debug output
for _, scheme in ipairs(wezterm.color.get_builtin_schemes()) do
  print(scheme)
end

return config
```

Then you can see the output by:
1. Running Wezterm from the terminal, which will show the debug output
2. Or by looking at Wezterm's debug overlay (press CTRL+SHIFT+L by default)

Alternatively, you can also use this shorter command in your terminal to list all schemes:
```bash
wezterm ls-colors
```

This will show you all available color schemes with their exact names as they should be used in the config file.

For the specific theme you mentioned, try using:
```lua
config.color_scheme = 'Summercamp'
```
or
```lua
config.color_scheme = 'summercamp'
```

The names are sometimes case-sensitive, so seeing the exact name from the list will help you use the correct one.

This is not working out for me

1. How would I run Wezterm from the terminal?
donf@MacBook-Air thag_rs % wezterm ls-colors
zsh: command not found: wezterm
"2. Or by looking at Wezterm's debug overlay (press CTRL+SHIFT+L by default)"
Debug Overlay
wezterm version: 20240203-110809-5046fc22 aarch64-apple-darwin
Window Environment: macOS 15.2 (24C101)
Lua Version: Lua 5.4
OpenGL: Apple M1 4.1 Metal - 89.3
Enter lua statements or expressions and hit Enter.
Press ESC or CTRL-D to exit
>

My ~/.wezterm.lua already had

local wezterm = require 'wezterm'

and

-- This will hold the configuration.
local config = wezterm.config_builder()

and

-- and finally, return the configuration to wezterm
return config

So i scratched my head and added the main bit just before `return config`:

-- This will print all available color schemes to the debug output
for _, scheme in ipairs(wezterm.color.get_builtin_schemes()) do
  print(scheme)
end

resulting in the above display, with no schemes printed.

Ah, I see the issue. Let me help you with a few things:

1. For the terminal command, you need to find where Wezterm is installed. On macOS, if you installed it via the .app bundle, you might need to add it to your PATH. The binary is typically located at:
```bash
/Applications/WezTerm.app/Contents/MacOS/wezterm
```
You can run it directly with that path, or add an alias to your shell config.

2. For listing the color schemes in the debug overlay (since you're already there), try entering this command directly in the debug overlay prompt:
```lua

```

3. Alternatively, you can modify your config file to explicitly show the schemes when Wezterm starts. Try this in your `~/.wezterm.lua`:

```lua
local wezterm = require 'wezterm'

local config = wezterm.config_builder()

-- Add this to show available schemes
wezterm.log_info(wezterm.color.get_builtin_schemes())

-- your other config settings here

return config
```

After making these changes, restart Wezterm and check the debug overlay (CTRL+SHIFT+L) - the schemes should be listed in the debug output at the top.

You can also add a function to your config to make it easier to search through schemes:
```lua
local wezterm = require 'wezterm'

local config = wezterm.config_builder()

-- Add this function to search schemes
config.keys = {
  {
    key = 'L',
    mods = 'CTRL|SHIFT',
    action = wezterm.action.ShowDebugOverlay,
  },
}

-- Print all schemes containing a specific string
local function find_schemes(pattern)
  for _, scheme in ipairs(wezterm.color.get_builtin_schemes()) do
    if scheme:lower():find(pattern:lower()) then
      wezterm.log_info(scheme)
    end
  end
end

-- Example: uncomment to search for "summer" schemes
-- find_schemes("summer")

return config
```

Thanks, I found the scheme and weirdly it was spelt correctly and eventually by dint of repetition is finally working. This also works but is too verbose to be practical: `wezterm.log_info(wezterm.color.get_builtin_schemes())`

I take it the config.keys is just meant to ensure the existing behaviour?

However, this does not work:

-- existing
local wezterm = require 'wezterm'

-- existing
local config = wezterm.config_builder()

...

local function find_schemes(pattern)
  for _, scheme in ipairs(wezterm.color.get_builtin_schemes()) do
    if scheme:lower():find(pattern:lower()) then
      wezterm.log_info(scheme)
    end
  end
end

-- Example: uncomment to search for "summer" schemes
find_schemes("summer")

-- existing
return config

Then:
Run Wezterm from Mac Spotlight

Ctrl-Shift-L:

Debug Overlay
wezterm version: 20240203-110809-5046fc22 aarch64-apple-darwin
Window Environment: macOS 15.2 (24C101)
Lua Version: Lua 5.4
OpenGL: Apple M1 4.1 Metal - 89.3
Enter lua statements or expressions and hit Enter.
Press ESC or CTRL-D to exit
>

The function and its invocation might as well not be there?!

Ah, I understand the confusion. The function definition and call need to be used in a specific way to see the output. Here are a few ways to make it work:

1. You can call the function directly in the debug overlay. After pressing Ctrl-Shift-L, type:
```lua
find_schemes("summer")
```

2. Or, to make it work in the config file, you need to call it during the configuration phase. Try this:

```lua
local wezterm = require 'wezterm'

local config = wezterm.config_builder()

local function find_schemes(pattern)
  for _, scheme in ipairs(wezterm.color.get_builtin_schemes()) do
    if scheme:lower():find(pattern:lower()) then
      wezterm.log_info(scheme)
    end
  end
end

-- Call it as part of the configuration setup
wezterm.on("gui-startup", function()
  find_schemes("summer")
end)

return config
```

And yes, the `config.keys` part I showed earlier was just documenting the existing default keybinding for the debug overlay (Ctrl-Shift-L). You don't need to include it unless you want to change the keybinding.

If you want a more immediate way to search schemes, you can also add this to your config:

```lua
local wezterm = require 'wezterm'

local config = wezterm.config_builder()

-- Make the function available globally
_G.find_schemes = function(pattern)
  for _, scheme in ipairs(wezterm.color.get_builtin_schemes()) do
    if scheme:lower():find(pattern:lower()) then
      wezterm.log_info(scheme)
    end
  end
end

return config
```

Then you can call it directly from the debug overlay (Ctrl-Shift-L) by typing:
```lua
find_schemes("summer")
```

This makes the function available in the debug overlay without having to define it each time.

Nothing to do with find_schemes("summer") is working. E.g. this as instructed

-- Make the function available globally
_G.find_schemes = function(pattern)
  for _, scheme in ipairs(wezterm.color.get_builtin_schemes()) do
    if scheme:lower():find(pattern:lower()) then
      wezterm.log_info(scheme)
    end
  end
end

Kill and restart Wezterm, and then Ctrl-Shift-L:

Debug Overlay
wezterm version: 20240203-110809-5046fc22 aarch64-apple-darwin
Window Environment: macOS 15.2 (24C101)
Lua Version: Lua 5.4
OpenGL: Apple M1 4.1 Metal - 89.3
Enter lua statements or expressions and hit Enter.
Press ESC or CTRL-D to exit
> find_schemes("cat")
> find_schemes("summer")
>

The best that can be said is that it seems to recognize the function and run it.



I see the issue now. Let's modify the function to use `print()` instead of `wezterm.log_info()` since we want to see the output directly in the debug overlay. Try this in your config:

```lua
_G.find_schemes = function(pattern)
  for _, scheme in ipairs(wezterm.color.get_builtin_schemes()) do
    if scheme:lower():find(pattern:lower()) then
      print(scheme)
    end
  end
end
```

Or, for a more foolproof approach that will show all matches at once:

```lua
_G.find_schemes = function(pattern)
  local matches = {}
  for _, scheme in ipairs(wezterm.color.get_builtin_schemes()) do
    if scheme:lower():find(pattern:lower()) then
      table.insert(matches, scheme)
    end
  end
  return table.concat(matches, "\n")
end
```

After adding either version to your config and restarting Wezterm, press Ctrl-Shift-L and try:
```lua
print(find_schemes("summer"))
```

This should now show the matching schemes directly in the debug overlay. The second version might be more reliable as it returns a string that you can then print or inspect.

Neither of these worked. The first didn't surprise me as a `print` was invoking a `print`. The second at least prints an empty string if I revert to omitting the print invocation:

Debug Overlay
wezterm version: 20240203-110809-5046fc22 aarch64-apple-darwin
Window Environment: macOS 15.2 (24C101)
Lua Version: Lua 5.4
OpenGL: Apple M1 4.1 Metal - 89.3
Enter lua statements or expressions and hit Enter.
Press ESC or CTRL-D to exit
> print(find_schemes("summer"))
21:40:05.145 INFO logging > lua:
> find_schemes("cat")
""
>
> print(find_schemes("AdventureTime"))
21:41:39.717 INFO logging > lua:
>

So it seems the find-schemes function is not working properly. It should at least find the existing full name AdventureTime.


Ah, now I see what's happening. Let's verify the behavior step by step. First, let's check if we can actually get the list of schemes. Try this in your config:

```lua
_G.find_schemes = function(pattern)
    -- First, let's see what schemes we actually have
    local schemes = wezterm.color.get_builtin_schemes()
    print("Number of schemes found: " .. #schemes)

    -- Print first few schemes to verify we're getting data
    for i = 1, math.min(5, #schemes) do
        print(schemes[i])
    end

    -- Now try to find matches
    local matches = {}
    for _, scheme in ipairs(schemes) do
        if scheme:lower():find(pattern:lower()) then
            table.insert(matches, scheme)
        end
    end

    return "Matches found: " .. #matches .. "\n" .. table.concat(matches, "\n")
end
```

After adding this to your config and restarting Wezterm, press Ctrl-Shift-L and try:
```lua
find_schemes("summer")
```

This should help us understand what's actually in the schemes list and if the search is working properly. If this doesn't show any schemes, we might need to try a different approach to access the color schemes.

Alternatively, we could try this simpler version just to verify we can access the schemes:

```lua
_G.list_schemes = function()
    local schemes = wezterm.color.get_built_in_schemes()
    return #schemes
end
```

Then in the debug overlay:
```lua
print(list_schemes())
```

This will at least tell us if we're able to access the schemes list at all.

Press ESC or CTRL-D to exit
> find_schemes("summer")
"Matches found: 0\n"
21:45:21.395 INFO logging > lua: Number of schemes found: 0
>
>
> print(list_schemes())
runtime error: [string "/Users/donf/.wezterm.lua"]:87: attempt to call a nil value (field 'get_built_in_schemes')
stack traceback:
        [string "/Users/donf/.wezterm.lua"]:87: in function 'list_schemes'
        [string "repl"]:1: in main chunk
>



Ah, there's the issue - I made a typo in the function name. The correct function is `get_builtin_schemes` (not `get_built_in_schemes`). However, it seems we're still not getting any schemes even with the correct function name.

Let's try a different approach. Instead of using `get_builtin_schemes`, we can use `get_scheme_names`. Try this:

```lua
_G.list_schemes = function()
    local schemes = wezterm.color.get_scheme_names()
    print("Number of schemes found: " .. #schemes)

    -- Print first few schemes to verify we're getting data
    for i = 1, math.min(5, #schemes) do
        print(schemes[i])
    end
    return schemes
end

_G.find_schemes = function(pattern)
    local schemes = wezterm.color.get_scheme_names()
    local matches = {}
    for _, scheme in ipairs(schemes) do
        if scheme:lower():find(pattern:lower()) then
            table.insert(matches, scheme)
        end
    end

    return "Matches found: " .. #matches .. "\n" .. table.concat(matches, "\n")
end
```

After adding this to your config and restarting Wezterm, try in the debug overlay:
```lua
list_schemes()
```
And then:
```lua
find_schemes("summer")
```

This should work better as `get_scheme_names()` is the more reliable way to get the list of available color schemes.

Press ESC or CTRL-D to exit
> list_schemes()
runtime error: [string "/Users/donf/.wezterm.lua"]:66: attempt to call a nil value (field 'get_scheme_names')
stack traceback:
        [string "/Users/donf/.wezterm.lua"]:66: in function 'list_schemes'
        (...tail calls...)
>
>
> find_schemes("summer")

runtime error: [string "/Users/donf/.wezterm.lua"]:77: attempt to call a nil value (field 'get_scheme_names')
stack traceback:
        [string "/Users/donf/.wezterm.lua"]:77: in function 'find_schemes'
        (...tail calls...)
>



Let me try one more approach. It seems the color scheme functions might be structured differently in the current version of Wezterm. Let's try this:

```lua
_G.list_schemes = function()
    local schemes = {}
    for name, _ in pairs(wezterm.color.get_schemes()) do
        table.insert(schemes, name)
    end
    print("Number of schemes found: " .. #schemes)

    -- Print first few schemes to verify we're getting data
    for i = 1, math.min(5, #schemes) do
        print(schemes[i])
    end
    return schemes
end

_G.find_schemes = function(pattern)
    local matches = {}
    for name, _ in pairs(wezterm.color.get_schemes()) do
        if name:lower():find(pattern:lower()) then
            table.insert(matches, name)
        end
    end

    return "Matches found: " .. #matches .. "\n" .. table.concat(matches, "\n")
end
```

Try in the debug overlay:
```lua
list_schemes()
```
And:
```lua
find_schemes("summer")
```

If this still doesn't work, could you try this simple test to see what methods are available:

```lua
_G.test_color = function()
    for k, v in pairs(wezterm.color) do
        print(k)
    end
end
```

Then in the debug overlay:
```lua
test_color()
```

This will help us see what functions are actually available in the color module.

The others still complained about `attempt to call a nil value (field 'get_schemes')`, but the simple test returned this:

Press ESC or CTRL-D to exit
> test_color()

21:51:21.002 INFO logging > lua: from_hsla
21:51:21.003 INFO logging > lua: gradient
21:51:21.003 INFO logging > lua: parse
21:51:21.003 INFO logging > lua: load_scheme
21:51:21.003 INFO logging > lua: load_base16_scheme
21:51:21.003 INFO logging > lua: extract_colors_from_image
21:51:21.003 INFO logging > lua: load_terminal_sexy_scheme
21:51:21.003 INFO logging > lua: save_scheme
21:51:21.004 INFO logging > lua: get_default_colors
21:51:21.004 INFO logging > lua: get_builtin_schemes
>


Ah, now we can see that `get_builtin_schemes` is indeed available. Let's try a modified version of our earlier code with proper error handling and logging:

```lua
_G.list_schemes = function()
    local schemes = wezterm.color.get_builtin_schemes()
    if type(schemes) ~= "table" then
        print("Error: schemes is not a table, it is: " .. type(schemes))
        return
    end

    print("Raw schemes data type: " .. type(schemes))
    print("Is it an array? " .. tostring(#schemes >= 0))

    -- Try to inspect the first element if it exists
    if #schemes > 0 then
        print("First scheme type: " .. type(schemes[1]))
        print("First scheme value: " .. tostring(schemes[1]))
    end

    return schemes
end

_G.find_schemes = function(pattern)
    local schemes = wezterm.color.get_builtin_schemes()
    if type(schemes) ~= "table" then
        return "Error: couldn't get schemes list"
    end

    local matches = {}
    for _, scheme in ipairs(schemes) do
        if tostring(scheme):lower():find(pattern:lower()) then
            table.insert(matches, tostring(scheme))
        end
    end

    return "Matches found: " .. #matches .. "\n" .. table.concat(matches, "\n")
end
```

Let's first try:
```lua
list_schemes()
```

This should give us more information about what we're actually getting from `get_builtin_schemes()`. Once we see that output, we can better understand how to properly search through the schemes.


list_schemes seems to have returned a full list:
...
"zenwritten_light": {
    "ansi": [
        "#eeeeee",
        "#a8334c",
        "#4f6c31",
        "#944927",
        "#286486",
        "#88507d",
        "#3b8992",
        "#353535",
    ],
    "background": "#eeeeee",
    "brights": [
        "#c6c3c3",
        "#94253e",
        "#3f5a22",
        "#803d1c",
        "#1d5573",
        "#7b3b70",
        "#2b747c",
        "#5c5c5c",
    ],
    "cursor_bg": "#353535",
    "cursor_border": "#353535",
    "cursor_fg": "#eeeeee",
    "foreground": "#353535",
    "indexed": [],
    "selection_bg": "#d7d7d7",
    "selection_fg": "#353535",
},
}
21:54:30.198 INFO logging > lua: Raw schemes data type: table
21:54:30.198 INFO logging > lua: Is it an array? true
> find_schemes("summer")
"Matches found: 0\n"
>

Ah, I see the issue now! The schemes are returned as a table where the keys are the scheme names, not as an array of names. Let's modify the `find_schemes` function to work with this structure:

```lua
_G.find_schemes = function(pattern)
    local schemes = wezterm.color.get_builtin_schemes()
    if type(schemes) ~= "table" then
        return "Error: couldn't get schemes list"
    end

    local matches = {}
    for name, _ in pairs(schemes) do
        if name:lower():find(pattern:lower()) then
            table.insert(matches, name)
        end
    end

    table.sort(matches)  -- Sort the matches alphabetically
    return "Matches found: " .. #matches .. "\n" .. table.concat(matches, "\n")
end
```

Now try:
```lua
find_schemes("summer")
```

This should work better because it's looking at the scheme names (the keys in the table) rather than trying to iterate over the table as an array. And since we saw "zenwritten_light" in your output, we can verify it works by trying:

```lua
find_schemes("zen")
```

This should find at least that scheme.

Success!

Press ESC or CTRL-D to exit
> find_schemes("summer")
"Matches found: 6\nEf-Summer\nSummer Pop (Gogh)\nSummerfruit Dark (base16)\nSummerfruit Light (base16)\nsummer-pop (Gogh)\nsummercamp (base16)"
>

And to answer my next question, the print() formats it nicely:

> print(find_schemes("summer"))
21:58:04.426 INFO logging > lua: Matches found: 6
Ef-Summer
Summer Pop (Gogh)
Summerfruit Dark (base16)
Summerfruit Light (base16)
summer-pop (Gogh)
summercamp (base16)
>
Thanks for your help, we got there in the end.


Yes, we did! And you discovered a nice detail about the `print()` function automatically formatting the newlines properly, which makes the output much more readable than the raw string representation.

To summarize what we learned:
1. The color schemes in Wezterm are stored as a table with scheme names as keys
2. We needed to use `pairs()` instead of `ipairs()` to iterate over the table
3. `wezterm.color.get_builtin_schemes()` is the correct function to get the schemes
4. Using `print()` gives nicer formatting for multi-line output in the debug overlay

Now you can easily search for color schemes by name, which should make it much easier to find and verify the correct scheme names for your config.

Thanks for your patience as we worked through the different approaches to get to the working solution!
