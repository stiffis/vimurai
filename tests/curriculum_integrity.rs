use vimurai::app::App;
use vimurai::app::screens::Screen;
use vimurai::engine::mode::VimMode;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, KeyEventKind, KeyEventState};

fn simulate_key(app: &mut App, c: char) {
    let code = match c {
        '\u{1b}' => KeyCode::Esc, // Escape hacks if needed
        c => KeyCode::Char(c),
    };

    let key = KeyEvent {
        code,
        modifiers: KeyModifiers::NONE,
        kind: KeyEventKind::Press,
        state: KeyEventState::NONE,
    };

    // Simulate the main event loop logic
    match app.practice_state.vim_mode {
        VimMode::Normal => app.handle_normal_mode(key).unwrap(),
        VimMode::Insert => app.handle_insert_mode(key).unwrap(),
        VimMode::Visual => app.handle_visual_mode(key).unwrap(),
        VimMode::Command => app.handle_command_mode(key).unwrap(),
        VimMode::OperatorPending(op) => app.handle_operator_pending_mode(key, op).unwrap(),
    }
}

#[test]
fn test_all_exercises_are_solvable() {
    let app_ref = App::new().unwrap();
    let exercises = app_ref.command_db.get_all_exercises();
    let mut failures = Vec::new();

    for exercise in exercises {
        println!("Testing Exercise: {} - {}", exercise.id, exercise.title);

        // 1. Setup fresh app state for this exercise
        let mut app = App::new().unwrap();
        app.current_screen = Screen::DailyDrill;
        
        // Manually load the exercise (simulating start_daily_drill logic)
        app.practice_state.vim_buffer.lines = exercise.initial_lines.clone();
        app.practice_state.vim_buffer.cursor_row = exercise.initial_cursor.0;
        app.practice_state.vim_buffer.cursor_col = exercise.initial_cursor.1;
        app.practice_state.current_exercise = Some(exercise.clone());
        app.practice_state.is_correct = None;

        // 2. Execute solution keys
        for key_char in exercise.solution_keys.chars() {
            simulate_key(&mut app, key_char);
        }

        // 3. Force a check (normally happens at end of handle_normal_mode)
        // We need to ensure the check runs even if we ended in Insert mode
        app.check_exercise_completion();

        // 4. Validate
                if app.practice_state.is_correct != Some(true) {
                    let error_msg = format!(
                        "FAILED: {} ({})\nExpected Lines: {:?}\nActual Lines:   {:?}\nExpected Cursor: {:?}\nActual Cursor:   {:?}",
                        exercise.id,
                        exercise.title,
                        exercise.expected_lines.as_ref().unwrap_or(&vec!["(Any)".to_string()]),
                        app.practice_state.vim_buffer.lines,
                        exercise.expected_cursor,
                        (app.practice_state.vim_buffer.cursor_row, app.practice_state.vim_buffer.cursor_col)
                    );
                    failures.push(error_msg);
                }
    }

    if !failures.is_empty() {
        panic!("Curriculum Integrity Check Failed:\n\n{}", failures.join("\n\n"));
    }
}
