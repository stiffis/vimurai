use vimurai::{
    curriculum::{exercises, validate_curriculum},
    editor::{Editor, EditorEvent, EditorKey},
};

fn parse_solution(notation: &str) -> Result<Vec<EditorKey>, String> {
    let mut keys = Vec::new();
    let mut chars = notation.chars().peekable();
    while let Some(character) = chars.next() {
        if character != '<' {
            keys.push(EditorKey::Char(character));
            continue;
        }
        let mut token = String::from("<");
        let mut closed = false;
        for next in chars.by_ref() {
            token.push(next);
            if next == '>' {
                closed = true;
                break;
            }
        }
        if !closed {
            return Err(format!("token sin cerrar en {notation:?}"));
        }
        let key = match token.as_str() {
            "<Esc>" => EditorKey::Esc,
            "<Enter>" => EditorKey::Enter,
            "<Tab>" => EditorKey::Tab,
            "<BS>" => EditorKey::Backspace,
            "<Del>" => EditorKey::Delete,
            "<C-d>" => EditorKey::Ctrl('d'),
            "<C-u>" => EditorKey::Ctrl('u'),
            "<C-f>" => EditorKey::Ctrl('f'),
            "<C-b>" => EditorKey::Ctrl('b'),
            _ => return Err(format!("token de solución desconocido: {token}")),
        };
        keys.push(key);
    }
    Ok(keys)
}

#[test]
fn every_reference_solution_reaches_its_declared_goal() {
    if let Err(errors) = validate_curriculum() {
        panic!("currículo inválido:\n{}", errors.join("\n"));
    }

    let mut failures = Vec::new();
    for exercise in exercises() {
        let keys = match parse_solution(exercise.solution) {
            Ok(keys) => keys,
            Err(error) => {
                failures.push(format!("{}: {error}", exercise.id));
                continue;
            }
        };
        let mut editor = Editor::new(
            exercise
                .initial_lines
                .iter()
                .map(|line| (*line).to_owned())
                .collect(),
            exercise.start,
        );
        let mut invalid = Vec::new();
        for key in keys {
            if let EditorEvent::Invalid { notation } = editor.handle_key(key, 12) {
                invalid.push(notation);
            }
        }
        if !invalid.is_empty() {
            failures.push(format!(
                "{}: la solución produjo comandos inválidos: {invalid:?}",
                exercise.id
            ));
        }
        if !exercise
            .goal
            .is_met(editor.lines(), editor.cursor(), editor.mode())
        {
            failures.push(format!(
                "{} ({}) no alcanzó la meta con {}: cursor={:?}, modo={:?}, buffer={:?}",
                exercise.id,
                exercise.title,
                exercise.solution,
                editor.cursor(),
                editor.mode(),
                editor.lines_as_strings(),
            ));
        }
    }

    assert!(
        failures.is_empty(),
        "fallaron soluciones curriculares:\n{}",
        failures.join("\n\n")
    );
}

#[test]
fn solution_dsl_recognizes_terminal_keys_as_single_events() {
    let keys = parse_solution("ihello<Esc><C-d>").unwrap();
    assert_eq!(keys.len(), 8);
    assert!(matches!(keys[6], EditorKey::Esc));
    assert!(matches!(keys[7], EditorKey::Ctrl('d')));
}
