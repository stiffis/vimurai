"""Refresh only Kage's generated half-block pixel body.

The Rust wrapper owns theme remapping, responsive art, state bubbles and tests;
this script deliberately leaves that hand-written code intact.
"""

from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
ART_PATH = ROOT / "scratch" / "cat_art.rs"
MASCOT_PATH = ROOT / "src" / "ui" / "mascot.rs"
START = "    // BEGIN GENERATED CAT PIXELS"
END = "    // END GENERATED CAT PIXELS"


def main() -> None:
    source = MASCOT_PATH.read_text(encoding="utf-8")
    art = ART_PATH.read_text(encoding="utf-8").strip()
    start = source.index(START)
    end = source.index(END, start) + len(END)
    replacement = f"{START}\n    let mut lines = {art};\n{END}"
    MASCOT_PATH.write_text(source[:start] + replacement + source[end:], encoding="utf-8")
    print("Updated Kage pixels; Gruvbox theming and responsive code were preserved.")


if __name__ == "__main__":
    main()
