/*[toml]
[dependencies]
thag_styling = { version = "0.2, thag-auto" }
*/

/// Debug example to check what ColorInitStrategy and theme are being used
///
/// This example helps diagnose why the integration modules might not be
/// getting the correct themed colors.
///
/// Run with:
/// ```bash
/// cargo run --example debug_theme_selection --features "color_detect"
/// cargo run --example debug_theme_selection --features "basic"
/// ```
use thag_styling::{paint_for_role, Role, Style, TermAttributes};

#[cfg(feature = "color_detect")]
use thag_styling::ColorInitStrategy;

#[cfg(feature = "color_detect")]
use thag_common::terminal;

fn main() {
    println!("🔍 Debug Theme Selection\n");

    // Test 1: Check what strategy would be determined
    #[cfg(feature = "color_detect")]
    {
        let strategy = ColorInitStrategy::determine();
        println!("1. ColorInitStrategy::determine() = {strategy:?}");
    }
    #[cfg(not(feature = "color_detect"))]
    {
        println!("1. ColorInitStrategy: color_detect feature not enabled");
    }

    // Test 1.5: Check raw color detection before initialization
    #[cfg(feature = "color_detect")]
    {
        println!("1.5. Raw color detection results:");
        let (color_support, term_bg_rgb) = terminal::detect_term_capabilities();
        println!("   - Raw color_support: {:?}", color_support);
        println!("   - Raw term_bg_rgb: {:?}", term_bg_rgb);

        let term_bg_luma = thag_common::terminal::is_light_color(*term_bg_rgb);
        println!("   - is_light_color: {}", term_bg_luma);
    }

    // Test 2: Initialize and check what we actually got
    let term_attrs = TermAttributes::get_or_init();
    println!("2. TermAttributes after initialization:");
    println!("   - how_initialized: {:?}", term_attrs.how_initialized);
    println!("   - color_support: {:?}", term_attrs.color_support);
    println!("   - term_bg_luma: {:?}", term_attrs.term_bg_luma);
    println!("   - theme.name: {}", term_attrs.theme.name);
    println!("   - theme.is_builtin: {}", term_attrs.theme.is_builtin);

    if let Some(rgb) = term_attrs.term_bg_rgb {
        println!("   - term_bg_rgb: RGB({}, {}, {})", rgb.0, rgb.1, rgb.2);
    } else {
        println!("   - term_bg_rgb: None");
    }

    // Test 3: Check what Style::from(Role) actually returns
    println!("\n3. Style::from(Role) analysis:");
    let roles = [
        Role::Success,
        Role::Error,
        Role::Warning,
        Role::Info,
        Role::Code,
    ];

    for role in roles {
        let style = Style::from(role);
        println!("   - {role:12}: foreground = {:?}", style.foreground);

        if let Some(color_info) = &style.foreground {
            println!("                   value = {:?}", color_info.value);
            println!("                   index = {}", color_info.index);
            println!(
                "                   ansi = {}",
                color_info.to_ansi_for_support(TermAttributes::get_or_init().color_support)
            );
        }
    }

    // Test 4: Check crossterm integration specifically
    #[cfg(feature = "crossterm_support")]
    {
        use crossterm::style::Color as CrossColor;

        println!("\n4. Crossterm integration test:");
        for role in [Role::Success, Role::Error, Role::Warning] {
            let crossterm_color = CrossColor::themed(role);
            println!("   - {role:12}: CrossColor = {crossterm_color:?}");
        }
    }

    // Test 5: Check ratatui integration specifically
    #[cfg(feature = "ratatui_support")]
    {
        use ratatui::style::Color as RataColor;

        println!("\n5. Ratatui integration test:");
        for role in [Role::Success, Role::Error, Role::Warning] {
            let ratatui_color = RataColor::themed(role);
            println!("   - {role:12}: RataColor = {ratatui_color:?}");
        }
    }

    // Test 6: Show the actual painted output
    println!("\n6. Actual painted output:");
    println!(
        "   Success: {}",
        paint_for_role(Role::Success, "Success message")
    );
    println!(
        "   Error:   {}",
        paint_for_role(Role::Error, "Error message")
    );
    println!(
        "   Warning: {}",
        paint_for_role(Role::Warning, "Warning message")
    );
    println!("   Info:    {}", paint_for_role(Role::Info, "Info message"));
    println!(
        "   Code:    {}",
        paint_for_role(Role::Code, "code_example()")
    );

    // Test 7: Force ColorInitStrategy::Match to see the difference
    #[cfg(feature = "color_detect")]
    {
        println!("\n7. Force ColorInitStrategy::Match test:");
        let match_attrs = TermAttributes::get_or_init_with_strategy(&ColorInitStrategy::Match);
        println!("   - how_initialized: {:?}", match_attrs.how_initialized);
        println!("   - color_support: {:?}", match_attrs.color_support);
        println!("   - theme.name: {}", match_attrs.theme.name);

        if let Some(rgb) = match_attrs.term_bg_rgb {
            println!(
                "   - Match term_bg_rgb: RGB({}, {}, {})",
                rgb.0, rgb.1, rgb.2
            );
        } else {
            println!("   - Match term_bg_rgb: None");
        }

        // Show styles from the Match strategy
        let success_style = match_attrs.theme.style_for(Role::Success);
        let error_style = match_attrs.theme.style_for(Role::Error);

        println!(
            "   Success style: foreground = {:?}",
            success_style.foreground
        );
        println!(
            "   Error style:   foreground = {:?}",
            error_style.foreground
        );
    }

    // Test 8: Force a TrueColor theme manually to see if that fixes integration colors
    #[cfg(feature = "color_detect")]
    {
        use thag_styling::{ColorSupport, Theme};

        println!("\n8. Manual TrueColor theme test:");

        // Try to load dracula theme with TrueColor support
        let dracula_result =
            Theme::get_theme_with_color_support("dracula", ColorSupport::TrueColor);
        match dracula_result {
            Ok(dracula_theme) => {
                println!("   Successfully loaded Dracula theme!");
                println!(
                    "   - min_color_support: {:?}",
                    dracula_theme.min_color_support
                );

                let success_style = dracula_theme.style_for(Role::Success);
                let error_style = dracula_theme.style_for(Role::Error);

                println!("   Dracula Success style: {:?}", success_style.foreground);
                println!("   Dracula Error style: {:?}", error_style.foreground);

                // Test crossterm integration with the better theme
                #[cfg(feature = "crossterm_support")]
                {
                    // Temporarily replace the global theme to test integration
                    println!("   Testing crossterm with Dracula colors...");
                    // Note: We can't easily test this without modifying global state
                }
            }
            Err(e) => {
                println!("   Failed to load Dracula theme: {}", e);
            }
        }

        // Try other popular themes
        let test_themes = [
            "nord",
            "gruvbox-dark-medium_base16",
            "solarized-dark_base16",
        ];
        for theme_name in test_themes {
            if let Ok(theme) =
                Theme::get_theme_with_color_support(theme_name, ColorSupport::TrueColor)
            {
                let success_style = theme.style_for(Role::Success);
                println!(
                    "   {}: Success = {:?}",
                    theme_name, success_style.foreground
                );
            }
        }
    }

    println!("\n✅ Debug complete!");
}
