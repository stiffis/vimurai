# Vimurai: the muscle memory dojo

Vimurai is a dojo for deliberate practice: it presents a concrete task using
realistic code, tracks how you solve it, and brings each skill back when it is
due for review.

This vision consolidates the project's original research and design drafts
into a concise reference for the current implementation.

## The four pillars

1. **Authentic practice.** A working mini Vim editor teaches the same cycle of
   modes, motions, and edits that learners can use in Vim or Neovim.
2. **Spaced repetition.** Daily Drill builds a finite queue for 3, 5, or 10
   minutes of practice. SM-2 schedules each concept using successes, mistakes,
   hints, and efficiency; a failed attempt provides useful information too.
3. **Lightweight gamification.** XP, belts, stars, streaks, activity, and
   achievements help learners track progress. They never make time a barrier
   to learning.
4. **Scaffolded progression.** Each belt introduces a few ideas before combining
   them: movement, precision, grammar, text objects, structural navigation,
   and reuse.

## The learning loop

```text
realistic goal → attempt in mini Vim → immediate feedback
      ↑                                      ↓
scheduled review ← semantic result + reflection
```

Efficiency is measured in semantic actions: `12j` counts as one action, as does
a completed search. A valid but inefficient solution passes and then invites
the learner to improve it.

## Practice modes and supporting views

- **Daily Drill:** personalized, bounded review; due concepts come first,
  followed by newly unlocked material and then mixed review.
- **Academy:** a belt-based campaign without time pressure, with objectives,
  context, progressive hints, and reference solutions.
- **Sandbox:** a free editor with Rust, Python, and text/log snippets; practice
  does not penalize progress.
- **Progress:** level, XP, accuracy, streak, activity heatmap, mastery, and
  achievements.
- **Reference and settings:** a searchable command catalog and local
  configuration.

## Belts

| Belt | Identity | Focus |
|---|---|---|
| White | Survivor | `hjkl`, words, and safe insertion |
| Yellow | Sniper | `fFtT`, line boundaries, and precision |
| Orange | Refactorer | Counts and operator + motion grammar |
| Red | Surgeon | Visual mode and text objects |
| Blue | Architect | Search, structure, and long-distance jumps |
| Black | Wizard | Registers, paste, and search repetition |

## Interaction principles

- Printable keys belong to the editor during practice. Help, restart, and exit
  use function keys or Ctrl combinations.
- Kage, the cat sensei, reacts to learning events. Its behavior requires no
  shortcuts that compete with Vim motions.
- The buffer keeps most of the available space. The interface shrinks or hides
  panels before reducing the practice area.
- The interface inherits the terminal's actual background and chooses Gruvbox
  Dark or Light based on its luminance, preserving the terminal's appearance.
- A terminal that is too small shows a safe resize prompt.
- Text coordinates consistently use Unicode scalar indices. Byte offsets and
  cursor columns are never mixed.
- All progress stays local. SQLite uses transactions, and tests use in-memory
  or temporary databases rather than a real user profile.

## Architecture

```text
terminal input
   ├─ application navigation → routes / overlays
   └─ active practice → Vim parser → typed action → editor
                                                 ├─ Kage feedback
                                                 ├─ exercise evaluation
                                                 └─ SM-2 + SQLite

immutable render state → responsive Ratatui components
```

The editor has no dependency on Ratatui. The curriculum is declarative and
can be validated independently. Persistence can be injected as an in-memory
store. Terminal cleanup uses RAII and a panic hook to restore raw mode, the
alternate screen, the cursor, and bracketed paste, including on panic paths.

## Scope

The local application prioritizes depth and correctness in its core learning
workflow. Social features, cloud services, AI, and plugins remain possible
extensions, provided they preserve the quality of the mini Vim editor, the
curriculum, and user privacy.
