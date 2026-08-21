use std::{
    io::{self, Stdout, Write},
    panic,
    sync::Once,
};

use crossterm::{
    ExecutableCommand,
    cursor::{Hide, SetCursorStyle, Show},
    event::{DisableBracketedPaste, EnableBracketedPaste},
    style::ResetColor,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Frame, Terminal, backend::CrosstermBackend, layout::Size};

pub type Backend = CrosstermBackend<Stdout>;

/// Owns every terminal mode changed by Vimurai.
///
/// Construction enters the alternate screen immediately. Cleanup can be
/// requested explicitly through [`restore`](Self::restore), and is also run on
/// drop as a best-effort fallback.
pub struct TerminalSession {
    /// Kept public for callers that need an advanced Ratatui operation. Prefer
    /// [`draw`](Self::draw), [`size`](Self::size), or
    /// [`terminal_mut`](Self::terminal_mut) for normal use.
    pub terminal: Terminal<Backend>,
    entered: bool,
}

impl TerminalSession {
    /// Creates a terminal and enters raw, alternate-screen mode.
    pub fn new() -> io::Result<Self> {
        install_panic_hook();

        let terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;
        let mut session = Self {
            terminal,
            entered: false,
        };
        session.enter()?;
        Ok(session)
    }

    /// Enters the managed terminal modes. Calling this while already entered
    /// is a no-op, which keeps compatibility with staged startup code.
    pub fn enter(&mut self) -> io::Result<()> {
        if self.entered {
            return Ok(());
        }

        // Mark the session first so a failure at any setup stage is followed
        // by a complete best-effort rollback.
        self.entered = true;

        let setup_result = (|| {
            enable_raw_mode()?;

            let backend = self.terminal.backend_mut();
            backend.execute(EnterAlternateScreen)?;
            backend.execute(ResetColor)?;
            backend.execute(Hide)?;
            backend.execute(EnableBracketedPaste)?;

            self.terminal.clear()?;
            Ok(())
        })();

        if let Err(error) = setup_result {
            let _ = self.restore();
            return Err(error);
        }

        Ok(())
    }

    /// Draws one complete Ratatui frame.
    pub fn draw<F>(&mut self, render: F) -> io::Result<()>
    where
        F: FnOnce(&mut Frame),
    {
        self.terminal.draw(render).map(|_| ())
    }

    /// Returns the physical terminal size currently reported by the backend.
    pub fn size(&self) -> io::Result<Size> {
        self.terminal.size()
    }

    pub const fn terminal(&self) -> &Terminal<Backend> {
        &self.terminal
    }

    pub const fn terminal_mut(&mut self) -> &mut Terminal<Backend> {
        &mut self.terminal
    }

    /// Emits the optional terminal bell without leaving Ratatui's screen.
    pub fn bell(&mut self) -> io::Result<()> {
        self.terminal.backend_mut().write_all(b"\x07")?;
        self.terminal.backend_mut().flush()
    }

    /// Restores every terminal mode Vimurai changes.
    ///
    /// Every cleanup operation is attempted even if an earlier one fails. The
    /// first error is returned after all steps have run. A second call is a
    /// no-op, making normal cleanup safely idempotent.
    pub fn restore(&mut self) -> io::Result<()> {
        if !self.entered {
            return Ok(());
        }

        self.entered = false;
        restore_terminal(self.terminal.backend_mut())
    }

    /// Backwards-compatible name for explicit restoration.
    pub fn exit(&mut self) -> io::Result<()> {
        self.restore()
    }
}

impl Drop for TerminalSession {
    fn drop(&mut self) {
        let _ = self.restore();
    }
}

/// Compatibility alias for code written against the initial prototype.
pub type Tui = TerminalSession;

/// Installs a process-wide panic hook that restores the terminal before
/// delegating to the hook that was installed previously.
pub fn install_panic_hook() {
    static INSTALL: Once = Once::new();

    INSTALL.call_once(|| {
        let previous_hook = panic::take_hook();
        panic::set_hook(Box::new(move |panic_info| {
            let mut stdout = io::stdout();
            let _ = restore_terminal(&mut stdout);
            previous_hook(panic_info);
        }));
    });
}

fn restore_terminal<W>(writer: &mut W) -> io::Result<()>
where
    W: Write,
{
    let mut first_error = None;

    remember_first(
        &mut first_error,
        writer.execute(SetCursorStyle::DefaultUserShape).map(|_| ()),
    );
    remember_first(&mut first_error, writer.execute(Show).map(|_| ()));
    remember_first(
        &mut first_error,
        writer.execute(DisableBracketedPaste).map(|_| ()),
    );
    remember_first(&mut first_error, writer.execute(ResetColor).map(|_| ()));
    remember_first(
        &mut first_error,
        writer.execute(LeaveAlternateScreen).map(|_| ()),
    );
    remember_first(&mut first_error, disable_raw_mode());

    match first_error {
        Some(error) => Err(error),
        None => Ok(()),
    }
}

fn remember_first(first_error: &mut Option<io::Error>, result: io::Result<()>) {
    if let Err(error) = result
        && first_error.is_none()
    {
        *first_error = Some(error);
    }
}
