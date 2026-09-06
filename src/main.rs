use std::{
    env,
    error::Error,
    process::ExitCode,
    time::{Duration, Instant},
};

use crossterm::event::Event;
use vimurai::{
    app::{App, AppOptions},
    terminal_appearance::TerminalInput,
    tui::TerminalSession,
    ui,
};

const TICK_RATE: Duration = Duration::from_millis(125);

#[derive(Debug, Default)]
struct Cli {
    ascii: bool,
    no_animation: bool,
    skip_boot: bool,
}

enum ParseResult {
    Run(Cli),
    Printed,
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("vimurai: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let ParseResult::Run(cli) = parse_args()? else {
        return Ok(());
    };
    // Open Termina while the shell is still cooked. It remains the sole input
    // reader, including after the bounded OSC 11 detection window.
    let mut input = TerminalInput::new()?;
    let mut terminal = TerminalSession::new()?;
    let detection = input.detect_theme();
    let mut app = App::new(AppOptions {
        skip_boot: cli.skip_boot,
        force_ascii: cli.ascii,
        no_animation: cli.no_animation,
        terminal_theme: detection.theme,
        theme_source: detection.source,
    })?;
    let mut next_tick = Instant::now() + TICK_RATE;

    while !app.should_quit {
        let size = terminal.size()?;
        app.set_viewport_height(usize::from(size.height.saturating_sub(12).max(3)));
        terminal.draw(|frame| ui::render(frame, &app))?;

        let timeout = next_tick.saturating_duration_since(Instant::now());
        let next_event = input.next_event(timeout)?;
        if let Some(event) = next_event {
            match event {
                Event::Key(key) => app.handle_key(key),
                Event::Paste(text) => app.handle_paste(&text),
                Event::Resize(_, _) | Event::FocusGained | Event::FocusLost | Event::Mouse(_) => {}
            }
        }
        let now = Instant::now();
        let mut caught_up = 0;
        while now >= next_tick && caught_up < 8 {
            app.on_tick();
            next_tick += TICK_RATE;
            caught_up += 1;
        }
        if now >= next_tick {
            next_tick = now + TICK_RATE;
        }
        if app.take_bell() {
            terminal.bell()?;
        }
    }

    // Guards unwind in reverse construction order: Crossterm first leaves its
    // modes, then Termina restores the shell state it captured before startup.
    terminal.restore()?;
    drop(input);
    if let Some(warning) = app.shutdown_warning.as_deref() {
        eprintln!("vimurai: no se pudo guardar toda la sesión: {warning}");
    }
    Ok(())
}

fn parse_args() -> Result<ParseResult, Box<dyn Error>> {
    let mut cli = Cli::default();
    for argument in env::args().skip(1) {
        match argument.as_str() {
            "--ascii" => cli.ascii = true,
            "--no-anim" | "--no-animation" => cli.no_animation = true,
            "--skip-boot" => cli.skip_boot = true,
            "-h" | "--help" => {
                println!(
                    "Vimurai {}\n\nUSO:\n    vimurai [OPCIONES]\n\nOPCIONES:\n    --ascii       fuerza arte ASCII monocromo\n    --no-anim     desactiva animaciones\n    --skip-boot   omite la secuencia inicial\n    -h, --help    muestra esta ayuda\n    -V, --version muestra la versión\n\nTEMA:\n    detecta Gruvbox Dark/Light y hereda el fondo del terminal\n    VIMURAI_THEME=dark|light fuerza una variante",
                    env!("CARGO_PKG_VERSION")
                );
                return Ok(ParseResult::Printed);
            }
            "-V" | "--version" => {
                println!("vimurai {}", env!("CARGO_PKG_VERSION"));
                return Ok(ParseResult::Printed);
            }
            unknown => return Err(format!("opción desconocida: {unknown}; usa --help").into()),
        }
    }
    Ok(ParseResult::Run(cli))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cli_defaults_are_accessible() {
        let cli = Cli::default();
        assert!(!cli.ascii);
        assert!(!cli.no_animation);
        assert!(!cli.skip_boot);
    }
}
