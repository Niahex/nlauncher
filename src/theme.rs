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
    pub cmd_process_bg: Hsla,    // ps - rouge avec 25% opacité
    pub cmd_process_text: Hsla,  // ps - rouge 100%
    pub cmd_password_bg: Hsla,   // pass - jaune avec 25% opacité
    pub cmd_password_text: Hsla, // pass - jaune 100%
    pub cmd_calc_bg: Hsla,       // = - vert avec 25% opacité
    pub cmd_calc_text: Hsla,     // = - vert 100%
    pub cmd_clip_bg: Hsla,       // clip - bleu avec 25% opacité
    pub cmd_clip_text: Hsla,     // clip - bleu 100%
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
            cmd_process_bg: rgba(0xbf616a40).into(), // rouge 25%
            cmd_process_text: rgb(0xbf616a).into(),  // rouge 100%
            cmd_password_bg: rgba(0xebcb8b40).into(), // jaune 25%
            cmd_password_text: rgb(0xebcb8b).into(), // jaune 100%
            cmd_calc_bg: rgba(0xa3be8c40).into(),    // vert 25%
            cmd_calc_text: rgb(0xa3be8c).into(),     // vert 100%
            cmd_clip_bg: rgba(0x81a1c140).into(),    // bleu 25%
            cmd_clip_text: rgb(0x81a1c1).into(),     // bleu 100%
        }
    }
}

impl NordTheme {
    pub fn new() -> Self {
        Self::default()
    }
}
