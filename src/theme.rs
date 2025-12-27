use gpui::{rgb, rgba, Hsla};

#[allow(dead_code)]
#[derive(Clone, Copy)]
pub struct NordTheme {
    // Polar Night (backgrounds)
    pub bg_primary: Hsla,
    pub bg_secondary: Hsla,
    pub bg_tertiary: Hsla,
    pub bg_elevated: Hsla,

    // Snow Storm (text)
    pub text_primary: Hsla,
    pub text_secondary: Hsla,
    pub text_muted: Hsla,

    // Frost (accents)
    pub accent_primary: Hsla,
    pub accent_secondary: Hsla,
    pub accent_tertiary: Hsla,
    pub accent_quaternary: Hsla,

    // Semantic colors
    pub success: Hsla,
    pub warning: Hsla,
    pub error: Hsla,
    pub info: Hsla,

    // Command-specific colors
    pub cmd_process_bg: Hsla,    // ps - red with 25% opacity
    pub cmd_process_text: Hsla,  // ps - red 100%
    pub cmd_password_bg: Hsla,   // pass - yellow with 25% opacity
    pub cmd_password_text: Hsla, // pass - yellow 100%
    pub cmd_calc_bg: Hsla,       // = - green with 25% opacity
    pub cmd_calc_text: Hsla,     // = - green 100%
    pub cmd_clip_bg: Hsla,       // clip - blue with 25% opacity
    pub cmd_clip_text: Hsla,     // clip - blue 100%
}

impl Default for NordTheme {
    fn default() -> Self {
        Self {
            // Polar Night
            bg_primary: rgb(0x2e3440).into(),
            bg_secondary: rgb(0x3b4252).into(),
            bg_tertiary: rgb(0x434c5e).into(),
            bg_elevated: rgb(0x4c566a).into(),

            // Snow Storm
            text_primary: rgb(0xeceff4).into(),
            text_secondary: rgb(0xe5e9f0).into(),
            text_muted: rgba(0xd8dee966).into(),

            // Frost
            accent_primary: rgb(0x88c0d0).into(),
            accent_secondary: rgb(0x81a1c1).into(),
            accent_tertiary: rgb(0x5e81ac).into(),
            accent_quaternary: rgb(0x8fbcbb).into(),

            // Semantic
            success: rgb(0xa3be8c).into(),
            warning: rgb(0xebcb8b).into(),
            error: rgb(0xbf616a).into(),
            info: rgb(0x81a1c1).into(),

            // Command-specific
            cmd_process_bg: rgba(0xbf616a40).into(), // red 25%
            cmd_process_text: rgb(0xbf616a).into(),  // red 100%
            cmd_password_bg: rgba(0xebcb8b40).into(), // yellow 25%
            cmd_password_text: rgb(0xebcb8b).into(), // yellow 100%
            cmd_calc_bg: rgba(0xa3be8c40).into(),    // green 25%
            cmd_calc_text: rgb(0xa3be8c).into(),     // green 100%
            cmd_clip_bg: rgba(0x81a1c140).into(),    // blue 25%
            cmd_clip_text: rgb(0x81a1c1).into(),     // blue 100%
        }
    }
}

impl NordTheme {
    pub fn new() -> Self {
        Self::default()
    }
}
