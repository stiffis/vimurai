//! Fast, best-effort terminal background detection.
//!
//! [`detect`] should be called after the terminal has entered raw mode and
//! before Crossterm's global event reader is used for the first time. In that
//! window Vimurai can ask for the default background with OSC 11 without
//! letting Crossterm mistake the reply for keyboard input.
//!
//! The probe has a short, bounded timeout. Real keyboard, paste, mouse, focus,
//! and resize events observed while waiting are returned in
//! [`ThemeDetection::pending_events`] and must be dispatched before reading the
//! next Crossterm event. If a probe cannot be made, environment hints are used
//! and the final safe fallback is [`TerminalTheme::Dark`].

use std::{env, io::Write as _, time::Duration};

use crossterm::event::{
    Event as CrosstermEvent, KeyCode as CrosstermKeyCode, KeyEvent as CrosstermKeyEvent,
    KeyEventKind as CrosstermKeyEventKind, KeyEventState as CrosstermKeyEventState,
    KeyModifiers as CrosstermKeyModifiers, MediaKeyCode as CrosstermMediaKeyCode,
    ModifierKeyCode as CrosstermModifierKeyCode, MouseButton as CrosstermMouseButton,
    MouseEvent as CrosstermMouseEvent, MouseEventKind as CrosstermMouseEventKind,
};
use termina::{
    Event as TerminaEvent, PlatformTerminal, Terminal as _,
    escape::osc::{ColorOrQuery, DynamicColorNumber, Osc},
    event::{
        KeyCode as TerminaKeyCode, KeyEventKind as TerminaKeyEventKind,
        MediaKeyCode as TerminaMediaKeyCode, ModifierKeyCode as TerminaModifierKeyCode,
        Modifiers as TerminaModifiers, MouseButton as TerminaMouseButton,
        MouseEventKind as TerminaMouseEventKind,
    },
};

/// Normal startup budget for an OSC 11 round trip.
pub const DEFAULT_PROBE_TIMEOUT: Duration = Duration::from_millis(50);

/// No caller can make startup wait longer than this for an optional hint.
const MAX_PROBE_TIMEOUT: Duration = Duration::from_millis(100);

/// The luminance where black and white have equal WCAG contrast.
const LIGHT_BACKGROUND_LUMINANCE: f64 = 0.179_128_784_747_792;

/// Upper bound that keeps draining pending input from becoming unbounded.
const MAX_PENDING_EVENTS: usize = 1_024;

/// Gruvbox variant matching the terminal's default background.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum TerminalTheme {
    /// Select Gruvbox Dark.
    #[default]
    Dark,
    /// Select Gruvbox Light.
    Light,
}

/// An RGB color reported by the terminal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rgb {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

impl Rgb {
    pub const fn new(red: u8, green: u8, blue: u8) -> Self {
        Self { red, green, blue }
    }
}

/// Why a terminal theme was selected.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum ThemeSource {
    /// An explicit environment override, with the variable that supplied it.
    Environment(&'static str),
    /// The terminal answered an OSC 11 default-background query.
    Osc11,
    /// The background index came from the conventional `COLORFGBG` variable.
    ColorFgBg,
    /// No reliable signal was available; the safe dark default was used.
    #[default]
    Default,
}

/// Theme selection plus diagnostics and input captured during the OSC query.
#[derive(Debug, Clone)]
pub struct ThemeDetection {
    pub theme: TerminalTheme,
    pub source: ThemeSource,
    /// Present when OSC 11 returned the actual default terminal background.
    pub background: Option<Rgb>,
    /// Dispatch these before calling `crossterm::event::poll` or `read`.
    pub pending_events: Vec<CrosstermEvent>,
}

impl Default for ThemeDetection {
    fn default() -> Self {
        Self {
            theme: TerminalTheme::Dark,
            source: ThemeSource::Default,
            background: None,
            pending_events: Vec::new(),
        }
    }
}

/// Detects the terminal theme with a 50 ms OSC 11 budget.
///
/// This function is infallible by design: terminal I/O failures simply move to
/// the next heuristic. See the module documentation for the required startup
/// ordering and how to replay [`ThemeDetection::pending_events`].
pub fn detect() -> ThemeDetection {
    detect_with_timeout(DEFAULT_PROBE_TIMEOUT)
}

/// Detects the terminal theme with a caller-supplied, bounded OSC 11 budget.
///
/// Values above 100 ms are capped so an optional terminal capability can never
/// make startup feel stuck. A zero timeout disables the OSC probe.
pub fn detect_with_timeout(timeout: Duration) -> ThemeDetection {
    if let Some((theme, variable)) = explicit_environment_override(env_value) {
        return ThemeDetection {
            theme,
            source: ThemeSource::Environment(variable),
            ..ThemeDetection::default()
        };
    }

    let environment_fallback = fallback_environment_hint(env_value);
    let timeout = timeout.min(MAX_PROBE_TIMEOUT);
    let (background, pending_events) = if !timeout.is_zero() && osc_probe_is_safe(env_value) {
        query_osc11(timeout)
    } else {
        (None, Vec::new())
    };

    if let Some(background) = background {
        return ThemeDetection {
            theme: theme_for_rgb(background),
            source: ThemeSource::Osc11,
            background: Some(background),
            pending_events,
        };
    }

    let (theme, source) =
        environment_fallback.unwrap_or((TerminalTheme::Dark, ThemeSource::Default));

    ThemeDetection {
        theme,
        source,
        background: None,
        pending_events,
    }
}

/// Parses an OSC 11 response such as `ESC ] 11;rgb:ffff/ffff/ffff ST`.
///
/// XParseColor permits one to four hexadecimal digits per channel. Each depth
/// is scaled to eight bits instead of merely taking the first byte.
pub fn parse_osc11_response(response: &[u8]) -> Option<Rgb> {
    const PREFIX: &[u8] = b"\x1b]11;";

    let start = response
        .windows(PREFIX.len())
        .position(|window| window == PREFIX)?
        + PREFIX.len();
    let tail = &response[start..];
    let bel = tail.iter().position(|byte| *byte == b'\x07');
    let string_terminator = tail.windows(2).position(|window| window == b"\x1b\\");
    let end = match (bel, string_terminator) {
        (Some(left), Some(right)) => left.min(right),
        (Some(end), None) | (None, Some(end)) => end,
        (None, None) => return None,
    };
    parse_x_color(&tail[..end])
}

/// Returns WCAG relative luminance in the inclusive range `0.0..=1.0`.
pub fn relative_luminance(rgb: Rgb) -> f64 {
    fn linear(channel: u8) -> f64 {
        let value = f64::from(channel) / 255.0;
        if value <= 0.040_45 {
            value / 12.92
        } else {
            ((value + 0.055) / 1.055).powf(2.4)
        }
    }

    0.2126 * linear(rgb.red) + 0.7152 * linear(rgb.green) + 0.0722 * linear(rgb.blue)
}

/// Chooses the variant whose foregrounds will contrast with `background`.
pub fn theme_for_rgb(background: Rgb) -> TerminalTheme {
    if relative_luminance(background) > LIGHT_BACKGROUND_LUMINANCE {
        TerminalTheme::Light
    } else {
        TerminalTheme::Dark
    }
}

/// Parses a conventional `COLORFGBG` value and classifies its final color.
///
/// Besides ANSI 0-15, xterm's 256-color cube and grayscale ramp are accepted.
pub fn parse_colorfgbg(value: &str) -> Option<TerminalTheme> {
    let index = value
        .rsplit([';', ':'])
        .find(|component| !component.trim().is_empty())?
        .trim()
        .parse::<u8>()
        .ok()?;
    Some(theme_for_rgb(ansi_index_rgb(index)))
}

/// Parses an explicit human-readable `dark`/`light` theme hint.
///
/// Separators are accepted (`gruvbox-dark`, `prefer_light`), while unrelated
/// words such as `highlight` do not accidentally count as `light`.
pub fn parse_theme_hint(value: &str) -> Option<TerminalTheme> {
    value
        .split(|character: char| !character.is_ascii_alphanumeric())
        .find_map(|word| match word.to_ascii_lowercase().as_str() {
            "dark" => Some(TerminalTheme::Dark),
            "light" => Some(TerminalTheme::Light),
            _ => None,
        })
}

/// Detects a theme from environment hints without doing terminal I/O.
pub fn detect_from_environment() -> Option<(TerminalTheme, ThemeSource)> {
    detect_environment_hint(env_value)
}

fn detect_environment_hint<F>(get: F) -> Option<(TerminalTheme, ThemeSource)>
where
    F: Copy + Fn(&str) -> Option<String>,
{
    if let Some((theme, variable)) = explicit_environment_override(get) {
        return Some((theme, ThemeSource::Environment(variable)));
    }
    fallback_environment_hint(get)
}

fn explicit_environment_override<F>(get: F) -> Option<(TerminalTheme, &'static str)>
where
    F: Copy + Fn(&str) -> Option<String>,
{
    ["VIMURAI_THEME", "VIMURAI_COLOR_SCHEME"]
        .into_iter()
        .find_map(|name| {
            get(name).and_then(|value| parse_theme_hint(&value).map(|theme| (theme, name)))
        })
}

fn fallback_environment_hint<F>(get: F) -> Option<(TerminalTheme, ThemeSource)>
where
    F: Copy + Fn(&str) -> Option<String>,
{
    ["TERM_BACKGROUND", "TERMINAL_THEME"]
        .into_iter()
        .find_map(|name| {
            get(name).and_then(|value| {
                parse_theme_hint(&value).map(|theme| (theme, ThemeSource::Environment(name)))
            })
        })
        .or_else(|| {
            get("COLORFGBG")
                .and_then(|value| parse_colorfgbg(&value))
                .map(|theme| (theme, ThemeSource::ColorFgBg))
        })
}

fn env_value(name: &str) -> Option<String> {
    env::var(name).ok()
}

fn osc_probe_is_safe<F>(get: F) -> bool
where
    F: Copy + Fn(&str) -> Option<String>,
{
    if !crossterm::terminal::is_raw_mode_enabled().unwrap_or(false) {
        return false;
    }
    if get("TERM").is_some_and(|term| term.eq_ignore_ascii_case("dumb")) {
        return false;
    }
    !get("VIMURAI_NO_OSC").is_some_and(|value| {
        matches!(
            value.trim().to_ascii_lowercase().as_str(),
            "1" | "true" | "yes"
        )
    })
}

fn query_osc11(timeout: Duration) -> (Option<Rgb>, Vec<CrosstermEvent>) {
    let Ok(mut terminal) = PlatformTerminal::new() else {
        return (None, Vec::new());
    };
    let query_result = write!(
        terminal,
        "{}",
        Osc::ChangeDynamicColors(
            DynamicColorNumber::TextBackgroundColor,
            vec![ColorOrQuery::Query]
        )
    )
    .and_then(|()| terminal.flush());
    if query_result.is_err() {
        return (None, drain_pending_events(&terminal));
    }

    let background = match terminal.poll(is_background_response, Some(timeout)) {
        Ok(true) => terminal
            .read(is_background_response)
            .ok()
            .and_then(|response| background_from_termina_event(&response)),
        Ok(false) | Err(_) => None,
    };
    let pending = drain_pending_events(&terminal);
    (background, pending)
}

fn is_background_response(event: &TerminaEvent) -> bool {
    background_from_termina_event(event).is_some()
}

fn background_from_termina_event(event: &TerminaEvent) -> Option<Rgb> {
    let TerminaEvent::Osc(Osc::ChangeDynamicColors(
        DynamicColorNumber::TextBackgroundColor,
        colors,
    )) = event
    else {
        return None;
    };

    colors.iter().find_map(|color| match color {
        ColorOrQuery::Color(color) => Some(Rgb::new(color.red, color.green, color.blue)),
        ColorOrQuery::Query => None,
    })
}

fn drain_pending_events(terminal: &PlatformTerminal) -> Vec<CrosstermEvent> {
    let mut pending = Vec::new();
    let mut drained = 0;
    while drained < MAX_PENDING_EVENTS {
        let Ok(true) = terminal.poll(|_| true, Some(Duration::ZERO)) else {
            break;
        };
        let Ok(event) = terminal.read(|_| true) else {
            break;
        };
        drained += 1;
        if let Some(event) = to_crossterm_event(event) {
            pending.push(event);
        }
    }
    pending
}

fn to_crossterm_event(event: TerminaEvent) -> Option<CrosstermEvent> {
    match event {
        TerminaEvent::Key(key) => Some(CrosstermEvent::Key(
            CrosstermKeyEvent::new_with_kind_and_state(
                to_crossterm_key_code(key.code),
                to_crossterm_modifiers(key.modifiers),
                match key.kind {
                    TerminaKeyEventKind::Press => CrosstermKeyEventKind::Press,
                    TerminaKeyEventKind::Repeat => CrosstermKeyEventKind::Repeat,
                    TerminaKeyEventKind::Release => CrosstermKeyEventKind::Release,
                },
                to_crossterm_state(key.state, key.modifiers),
            ),
        )),
        TerminaEvent::Mouse(mouse) => Some(CrosstermEvent::Mouse(CrosstermMouseEvent {
            kind: to_crossterm_mouse_kind(mouse.kind),
            column: mouse.column,
            row: mouse.row,
            modifiers: to_crossterm_modifiers(mouse.modifiers),
        })),
        TerminaEvent::WindowResized(size) => Some(CrosstermEvent::Resize(size.cols, size.rows)),
        TerminaEvent::FocusIn => Some(CrosstermEvent::FocusGained),
        TerminaEvent::FocusOut => Some(CrosstermEvent::FocusLost),
        TerminaEvent::Paste(text) => Some(CrosstermEvent::Paste(text)),
        TerminaEvent::Csi(_) | TerminaEvent::Osc(_) | TerminaEvent::Dcs(_) => None,
    }
}

fn to_crossterm_key_code(code: TerminaKeyCode) -> CrosstermKeyCode {
    match code {
        TerminaKeyCode::Char(character) => CrosstermKeyCode::Char(character),
        TerminaKeyCode::Enter => CrosstermKeyCode::Enter,
        TerminaKeyCode::Backspace => CrosstermKeyCode::Backspace,
        TerminaKeyCode::Tab => CrosstermKeyCode::Tab,
        TerminaKeyCode::Escape => CrosstermKeyCode::Esc,
        TerminaKeyCode::Left => CrosstermKeyCode::Left,
        TerminaKeyCode::Right => CrosstermKeyCode::Right,
        TerminaKeyCode::Up => CrosstermKeyCode::Up,
        TerminaKeyCode::Down => CrosstermKeyCode::Down,
        TerminaKeyCode::Home => CrosstermKeyCode::Home,
        TerminaKeyCode::End => CrosstermKeyCode::End,
        TerminaKeyCode::BackTab => CrosstermKeyCode::BackTab,
        TerminaKeyCode::PageUp => CrosstermKeyCode::PageUp,
        TerminaKeyCode::PageDown => CrosstermKeyCode::PageDown,
        TerminaKeyCode::Insert => CrosstermKeyCode::Insert,
        TerminaKeyCode::Delete => CrosstermKeyCode::Delete,
        TerminaKeyCode::KeypadBegin => CrosstermKeyCode::KeypadBegin,
        TerminaKeyCode::CapsLock => CrosstermKeyCode::CapsLock,
        TerminaKeyCode::ScrollLock => CrosstermKeyCode::ScrollLock,
        TerminaKeyCode::NumLock => CrosstermKeyCode::NumLock,
        TerminaKeyCode::PrintScreen => CrosstermKeyCode::PrintScreen,
        TerminaKeyCode::Pause => CrosstermKeyCode::Pause,
        TerminaKeyCode::Menu => CrosstermKeyCode::Menu,
        TerminaKeyCode::Null => CrosstermKeyCode::Null,
        TerminaKeyCode::Function(number) => CrosstermKeyCode::F(number),
        TerminaKeyCode::Modifier(modifier) => {
            CrosstermKeyCode::Modifier(to_crossterm_modifier_key(modifier))
        }
        TerminaKeyCode::Media(media) => CrosstermKeyCode::Media(to_crossterm_media_key(media)),
    }
}

fn to_crossterm_modifiers(modifiers: TerminaModifiers) -> CrosstermKeyModifiers {
    let mut result = CrosstermKeyModifiers::NONE;
    result.set(
        CrosstermKeyModifiers::SHIFT,
        modifiers.contains(TerminaModifiers::SHIFT),
    );
    result.set(
        CrosstermKeyModifiers::ALT,
        modifiers.contains(TerminaModifiers::ALT),
    );
    result.set(
        CrosstermKeyModifiers::CONTROL,
        modifiers.contains(TerminaModifiers::CONTROL),
    );
    result.set(
        CrosstermKeyModifiers::SUPER,
        modifiers.contains(TerminaModifiers::SUPER),
    );
    result.set(
        CrosstermKeyModifiers::HYPER,
        modifiers.contains(TerminaModifiers::HYPER),
    );
    result.set(
        CrosstermKeyModifiers::META,
        modifiers.contains(TerminaModifiers::META),
    );
    result
}

fn to_crossterm_state(
    state: termina::event::KeyEventState,
    modifiers: TerminaModifiers,
) -> CrosstermKeyEventState {
    let mut result = CrosstermKeyEventState::NONE;
    result.set(
        CrosstermKeyEventState::KEYPAD,
        state.contains(termina::event::KeyEventState::KEYPAD),
    );
    result.set(
        CrosstermKeyEventState::CAPS_LOCK,
        state.contains(termina::event::KeyEventState::CAPS_LOCK)
            || modifiers.contains(TerminaModifiers::CAPS_LOCK),
    );
    result.set(
        CrosstermKeyEventState::NUM_LOCK,
        state.contains(termina::event::KeyEventState::NUM_LOCK)
            || modifiers.contains(TerminaModifiers::NUM_LOCK),
    );
    result
}

fn to_crossterm_mouse_kind(kind: TerminaMouseEventKind) -> CrosstermMouseEventKind {
    match kind {
        TerminaMouseEventKind::Down(button) => {
            CrosstermMouseEventKind::Down(to_crossterm_mouse_button(button))
        }
        TerminaMouseEventKind::Up(button) => {
            CrosstermMouseEventKind::Up(to_crossterm_mouse_button(button))
        }
        TerminaMouseEventKind::Drag(button) => {
            CrosstermMouseEventKind::Drag(to_crossterm_mouse_button(button))
        }
        TerminaMouseEventKind::Moved => CrosstermMouseEventKind::Moved,
        TerminaMouseEventKind::ScrollDown => CrosstermMouseEventKind::ScrollDown,
        TerminaMouseEventKind::ScrollUp => CrosstermMouseEventKind::ScrollUp,
        TerminaMouseEventKind::ScrollLeft => CrosstermMouseEventKind::ScrollLeft,
        TerminaMouseEventKind::ScrollRight => CrosstermMouseEventKind::ScrollRight,
    }
}

fn to_crossterm_mouse_button(button: TerminaMouseButton) -> CrosstermMouseButton {
    match button {
        TerminaMouseButton::Left => CrosstermMouseButton::Left,
        TerminaMouseButton::Right => CrosstermMouseButton::Right,
        TerminaMouseButton::Middle => CrosstermMouseButton::Middle,
    }
}

fn to_crossterm_modifier_key(modifier: TerminaModifierKeyCode) -> CrosstermModifierKeyCode {
    match modifier {
        TerminaModifierKeyCode::LeftShift => CrosstermModifierKeyCode::LeftShift,
        TerminaModifierKeyCode::LeftControl => CrosstermModifierKeyCode::LeftControl,
        TerminaModifierKeyCode::LeftAlt => CrosstermModifierKeyCode::LeftAlt,
        TerminaModifierKeyCode::LeftSuper => CrosstermModifierKeyCode::LeftSuper,
        TerminaModifierKeyCode::LeftHyper => CrosstermModifierKeyCode::LeftHyper,
        TerminaModifierKeyCode::LeftMeta => CrosstermModifierKeyCode::LeftMeta,
        TerminaModifierKeyCode::RightShift => CrosstermModifierKeyCode::RightShift,
        TerminaModifierKeyCode::RightControl => CrosstermModifierKeyCode::RightControl,
        TerminaModifierKeyCode::RightAlt => CrosstermModifierKeyCode::RightAlt,
        TerminaModifierKeyCode::RightSuper => CrosstermModifierKeyCode::RightSuper,
        TerminaModifierKeyCode::RightHyper => CrosstermModifierKeyCode::RightHyper,
        TerminaModifierKeyCode::RightMeta => CrosstermModifierKeyCode::RightMeta,
        TerminaModifierKeyCode::IsoLevel3Shift => CrosstermModifierKeyCode::IsoLevel3Shift,
        TerminaModifierKeyCode::IsoLevel5Shift => CrosstermModifierKeyCode::IsoLevel5Shift,
    }
}

fn to_crossterm_media_key(media: TerminaMediaKeyCode) -> CrosstermMediaKeyCode {
    match media {
        TerminaMediaKeyCode::Play => CrosstermMediaKeyCode::Play,
        TerminaMediaKeyCode::Pause => CrosstermMediaKeyCode::Pause,
        TerminaMediaKeyCode::PlayPause => CrosstermMediaKeyCode::PlayPause,
        TerminaMediaKeyCode::Reverse => CrosstermMediaKeyCode::Reverse,
        TerminaMediaKeyCode::Stop => CrosstermMediaKeyCode::Stop,
        TerminaMediaKeyCode::FastForward => CrosstermMediaKeyCode::FastForward,
        TerminaMediaKeyCode::Rewind => CrosstermMediaKeyCode::Rewind,
        TerminaMediaKeyCode::TrackNext => CrosstermMediaKeyCode::TrackNext,
        TerminaMediaKeyCode::TrackPrevious => CrosstermMediaKeyCode::TrackPrevious,
        TerminaMediaKeyCode::Record => CrosstermMediaKeyCode::Record,
        TerminaMediaKeyCode::LowerVolume => CrosstermMediaKeyCode::LowerVolume,
        TerminaMediaKeyCode::RaiseVolume => CrosstermMediaKeyCode::RaiseVolume,
        TerminaMediaKeyCode::MuteVolume => CrosstermMediaKeyCode::MuteVolume,
    }
}

fn parse_x_color(value: &[u8]) -> Option<Rgb> {
    if let Some(value) = value.strip_prefix(b"rgb:") {
        let mut channels = value.split(|byte| *byte == b'/');
        let red = parse_hex_channel(channels.next()?)?;
        let green = parse_hex_channel(channels.next()?)?;
        let blue = parse_hex_channel(channels.next()?)?;
        if channels.next().is_some() {
            return None;
        }
        return Some(Rgb::new(red, green, blue));
    }

    let hex = value.strip_prefix(b"#")?;
    let channel_width = match hex.len() {
        3 => 1,
        6 => 2,
        9 => 3,
        12 => 4,
        _ => return None,
    };
    Some(Rgb::new(
        parse_hex_channel(&hex[..channel_width])?,
        parse_hex_channel(&hex[channel_width..channel_width * 2])?,
        parse_hex_channel(&hex[channel_width * 2..])?,
    ))
}

fn parse_hex_channel(channel: &[u8]) -> Option<u8> {
    if channel.is_empty() || channel.len() > 4 || !channel.is_ascii() {
        return None;
    }
    let value = u16::from_str_radix(std::str::from_utf8(channel).ok()?, 16).ok()?;
    let maximum = (1u32 << (channel.len() * 4)) - 1;
    Some(((u32::from(value) * 255 + maximum / 2) / maximum) as u8)
}

fn ansi_index_rgb(index: u8) -> Rgb {
    const ANSI: [Rgb; 16] = [
        Rgb::new(0, 0, 0),
        Rgb::new(128, 0, 0),
        Rgb::new(0, 128, 0),
        Rgb::new(128, 128, 0),
        Rgb::new(0, 0, 128),
        Rgb::new(128, 0, 128),
        Rgb::new(0, 128, 128),
        Rgb::new(192, 192, 192),
        Rgb::new(128, 128, 128),
        Rgb::new(255, 0, 0),
        Rgb::new(0, 255, 0),
        Rgb::new(255, 255, 0),
        Rgb::new(0, 0, 255),
        Rgb::new(255, 0, 255),
        Rgb::new(0, 255, 255),
        Rgb::new(255, 255, 255),
    ];
    const CUBE: [u8; 6] = [0, 95, 135, 175, 215, 255];

    match index {
        0..=15 => ANSI[usize::from(index)],
        16..=231 => {
            let offset = index - 16;
            Rgb::new(
                CUBE[usize::from(offset / 36)],
                CUBE[usize::from((offset % 36) / 6)],
                CUBE[usize::from(offset % 6)],
            )
        }
        232..=255 => {
            let gray = 8 + (index - 232) * 10;
            Rgb::new(gray, gray, gray)
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crossterm::event::{KeyCode, KeyModifiers};
    use termina::Parser;

    use super::*;

    #[test]
    fn dark_is_the_safe_default() {
        assert_eq!(TerminalTheme::default(), TerminalTheme::Dark);
        assert_eq!(ThemeDetection::default().source, ThemeSource::Default);
    }

    #[test]
    fn parses_common_osc11_depths_and_terminators() {
        assert_eq!(
            parse_osc11_response(b"\x1b]11;rgb:28/28/28\x07"),
            Some(Rgb::new(0x28, 0x28, 0x28))
        );
        assert_eq!(
            parse_osc11_response(b"noise\x1b]11;rgb:ffff/f1f1/c7c7\x1b\\tail"),
            Some(Rgb::new(255, 241, 199))
        );
        assert_eq!(
            parse_osc11_response(b"\x1b]11;rgb:f/0/8\x07"),
            Some(Rgb::new(255, 0, 136))
        );
        assert_eq!(
            parse_osc11_response(b"\x1b]11;#fbf1c7\x07"),
            Some(Rgb::new(0xfb, 0xf1, 0xc7))
        );
    }

    #[test]
    fn rejects_incomplete_or_wrong_osc_responses() {
        assert_eq!(parse_osc11_response(b"\x1b]10;rgb:ff/ff/ff\x07"), None);
        assert_eq!(parse_osc11_response(b"\x1b]11;rgb:ff/ff\x07"), None);
        assert_eq!(parse_osc11_response(b"\x1b]11;rgb:gg/ff/ff\x07"), None);
        assert_eq!(parse_osc11_response(b"\x1b]11;rgb:ff/ff/ff"), None);
    }

    #[test]
    fn classifies_both_gruvbox_backgrounds() {
        let dark = Rgb::new(0x28, 0x28, 0x28);
        let light = Rgb::new(0xfb, 0xf1, 0xc7);
        assert_eq!(theme_for_rgb(dark), TerminalTheme::Dark);
        assert_eq!(theme_for_rgb(light), TerminalTheme::Light);
        assert!(relative_luminance(dark) < relative_luminance(light));
    }

    #[test]
    fn parses_colorfgbg_and_xterm_indices() {
        assert_eq!(parse_colorfgbg("15;0"), Some(TerminalTheme::Dark));
        assert_eq!(parse_colorfgbg("0;15"), Some(TerminalTheme::Light));
        assert_eq!(parse_colorfgbg("0;15;"), Some(TerminalTheme::Light));
        assert_eq!(parse_colorfgbg("7:232"), Some(TerminalTheme::Dark));
        assert_eq!(parse_colorfgbg("0;255"), Some(TerminalTheme::Light));
        assert_eq!(parse_colorfgbg("unknown"), None);
    }

    #[test]
    fn parses_theme_words_without_light_substring_false_positive() {
        assert_eq!(parse_theme_hint("gruvbox-dark"), Some(TerminalTheme::Dark));
        assert_eq!(parse_theme_hint("prefer_light"), Some(TerminalTheme::Light));
        assert_eq!(parse_theme_hint("highlight"), None);
    }

    #[test]
    fn environment_order_prefers_vimurai_override() {
        let values = HashMap::from([
            ("VIMURAI_THEME", "light"),
            ("TERM_BACKGROUND", "dark"),
            ("COLORFGBG", "15;0"),
        ]);
        let result = detect_environment_hint(|name| values.get(name).map(ToString::to_string));
        assert_eq!(
            result,
            Some((
                TerminalTheme::Light,
                ThemeSource::Environment("VIMURAI_THEME")
            ))
        );
    }

    #[test]
    fn termina_parser_separates_osc_reply_from_real_input() {
        let mut parser = Parser::default();
        parser.parse(b"x\x1b]11;rgb:2828/2828/2828\x1b\\", false);

        let key = parser.pop().expect("key event");
        let response = parser.pop().expect("OSC response");
        assert_eq!(
            to_crossterm_event(key),
            Some(CrosstermEvent::Key(CrosstermKeyEvent::new(
                KeyCode::Char('x'),
                KeyModifiers::NONE
            )))
        );
        assert_eq!(
            background_from_termina_event(&response),
            Some(Rgb::new(0x28, 0x28, 0x28))
        );
    }
}
