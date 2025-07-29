//! User interface utilities for thag_profiler
//!
//! This module provides UI-related functionality for thag_profiler tools,
//! including theme-aware styling for inquire prompts when available.

#[cfg(feature = "inquire_theming")]
pub mod inquire_theming;

#[cfg(not(feature = "inquire_theming"))]
pub mod inquire_theming {
    //! Fallback inquire theming module when feature is disabled

    /// Fallback function when theming is disabled - returns unit type
    pub fn get_themed_render_config() -> () {
        ()
    }

    /// No-op fallback when theming is disabled
    pub fn apply_global_theming() {
        // Do nothing when theming is not available
    }

    /// Fallback terminal info when theming is disabled
    pub fn get_terminal_info() -> ((), ()) {
        ((), ())
    }
}
