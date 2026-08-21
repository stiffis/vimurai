use ratatui::style::{Color, Modifier, Style};

use crate::terminal_appearance::TerminalTheme;

/// Gruvbox colors adapted to a terminal UI that inherits the user's own
/// background. `background`, `panel`, and `elevated` deliberately remain
/// `Color::Reset`: Vimurai never paints an opaque application canvas.
#[derive(Debug, Clone, Copy)]
pub struct Theme {
    pub variant: TerminalTheme,
    pub background: Color,
    pub panel: Color,
    pub elevated: Color,
    /// Readable foreground for cells with a semantic accent background.
    pub on_accent: Color,
    pub primary: Color,
    pub cyan: Color,
    pub amber: Color,
    pub danger: Color,
    pub violet: Color,
    pub orange: Color,
    pub blue: Color,
    pub text: Color,
    pub muted: Color,
    pub border: Color,
    pub mascot_outline: Color,
    pub mascot_fur: Color,
    pub mascot_highlight: Color,
    pub mascot_shade: Color,
    pub mascot_accent: Color,
}

impl Theme {
    #[must_use]
    pub const fn gruvbox(variant: TerminalTheme, high_contrast: bool, no_color: bool) -> Self {
        if no_color {
            return Self::monochrome(variant);
        }

        match variant {
            TerminalTheme::Dark => Self {
                variant,
                background: Color::Reset,
                panel: Color::Reset,
                elevated: Color::Reset,
                on_accent: Color::Rgb(40, 40, 40), // bg0
                primary: Color::Rgb(184, 187, 38), // bright green
                cyan: Color::Rgb(142, 192, 124),   // bright aqua
                amber: Color::Rgb(250, 189, 47),   // bright yellow
                danger: Color::Rgb(251, 73, 52),   // bright red
                violet: Color::Rgb(211, 134, 155), // bright purple
                orange: Color::Rgb(254, 128, 25),  // bright orange
                blue: Color::Rgb(131, 165, 152),   // bright blue
                text: if high_contrast {
                    Color::Rgb(251, 241, 199) // fg0
                } else {
                    Color::Rgb(235, 219, 178) // fg1
                },
                muted: if high_contrast {
                    Color::Rgb(213, 196, 161) // fg2
                } else {
                    Color::Rgb(168, 153, 132) // fg4
                },
                border: if high_contrast {
                    Color::Rgb(184, 187, 38)
                } else {
                    Color::Rgb(102, 92, 84) // bg3
                },
                mascot_outline: Color::Rgb(40, 40, 40),
                mascot_fur: Color::Rgb(250, 189, 47),
                mascot_highlight: Color::Rgb(251, 241, 199),
                mascot_shade: Color::Rgb(254, 128, 25),
                mascot_accent: Color::Rgb(251, 73, 52),
            },
            TerminalTheme::Light => Self {
                variant,
                background: Color::Reset,
                panel: Color::Reset,
                elevated: Color::Reset,
                on_accent: Color::Rgb(251, 241, 199), // bg0
                primary: Color::Rgb(121, 116, 14),    // green
                cyan: Color::Rgb(66, 123, 88),        // aqua
                amber: Color::Rgb(181, 118, 20),      // yellow
                danger: Color::Rgb(157, 0, 6),        // red
                violet: Color::Rgb(143, 63, 113),     // purple
                orange: Color::Rgb(175, 58, 3),       // orange
                blue: Color::Rgb(7, 102, 120),        // blue
                text: if high_contrast {
                    Color::Rgb(40, 40, 40) // fg0
                } else {
                    Color::Rgb(60, 56, 54) // fg1
                },
                muted: if high_contrast {
                    Color::Rgb(80, 73, 69) // fg2
                } else {
                    Color::Rgb(124, 111, 100) // fg4
                },
                border: if high_contrast {
                    Color::Rgb(121, 116, 14)
                } else {
                    Color::Rgb(189, 174, 147) // bg3
                },
                mascot_outline: Color::Rgb(60, 56, 54),
                mascot_fur: Color::Rgb(181, 118, 20),
                mascot_highlight: Color::Rgb(215, 153, 33),
                mascot_shade: Color::Rgb(175, 58, 3),
                mascot_accent: Color::Rgb(157, 0, 6),
            },
        }
    }

    const fn monochrome(variant: TerminalTheme) -> Self {
        Self {
            variant,
            background: Color::Reset,
            panel: Color::Reset,
            elevated: Color::Reset,
            on_accent: Color::Reset,
            primary: Color::Reset,
            cyan: Color::Reset,
            amber: Color::Reset,
            danger: Color::Reset,
            violet: Color::Reset,
            orange: Color::Reset,
            blue: Color::Reset,
            text: Color::Reset,
            muted: Color::Reset,
            border: Color::Reset,
            mascot_outline: Color::Reset,
            mascot_fur: Color::Reset,
            mascot_highlight: Color::Reset,
            mascot_shade: Color::Reset,
            mascot_accent: Color::Reset,
        }
    }

    #[must_use]
    pub const fn label(self) -> &'static str {
        match self.variant {
            TerminalTheme::Dark => "GRUVBOX DARK",
            TerminalTheme::Light => "GRUVBOX LIGHT",
        }
    }

    #[must_use]
    pub fn title(self) -> Style {
        Style::default()
            .fg(self.primary)
            .add_modifier(Modifier::BOLD)
    }

    #[must_use]
    pub fn focused(self) -> Style {
        Style::default().fg(self.cyan).add_modifier(Modifier::BOLD)
    }

    #[must_use]
    pub fn dim(self) -> Style {
        Style::default().fg(self.muted)
    }

    #[must_use]
    pub fn selected(self) -> Style {
        Style::default()
            .fg(self.on_accent)
            .bg(self.primary)
            .add_modifier(Modifier::BOLD)
    }

    #[must_use]
    pub fn target(self) -> Style {
        Style::default()
            .fg(self.on_accent)
            .bg(self.amber)
            .add_modifier(Modifier::BOLD)
    }

    #[must_use]
    pub fn error(self) -> Style {
        Style::default()
            .fg(self.danger)
            .add_modifier(Modifier::BOLD)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn both_gruvbox_variants_inherit_every_surface() {
        for variant in [TerminalTheme::Dark, TerminalTheme::Light] {
            let theme = Theme::gruvbox(variant, false, false);
            assert_eq!(theme.background, Color::Reset);
            assert_eq!(theme.panel, Color::Reset);
            assert_eq!(theme.elevated, Color::Reset);
        }
    }

    #[test]
    fn dark_and_light_use_opposite_readable_ink() {
        let dark = Theme::gruvbox(TerminalTheme::Dark, false, false);
        let light = Theme::gruvbox(TerminalTheme::Light, false, false);
        assert_eq!(dark.text, Color::Rgb(235, 219, 178));
        assert_eq!(light.text, Color::Rgb(60, 56, 54));
        assert_ne!(dark.on_accent, light.on_accent);
        assert_eq!(dark.label(), "GRUVBOX DARK");
        assert_eq!(light.label(), "GRUVBOX LIGHT");
    }

    #[test]
    fn no_color_really_uses_terminal_defaults() {
        let theme = Theme::gruvbox(TerminalTheme::Light, true, true);
        assert_eq!(theme.text, Color::Reset);
        assert_eq!(theme.primary, Color::Reset);
        assert_eq!(theme.selected().fg, Some(Color::Reset));
        assert_eq!(theme.selected().bg, Some(Color::Reset));
    }
}
