#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Snippet {
    pub name: &'static str,
    pub language: &'static str,
    pub lines: &'static [&'static str],
}

pub static SNIPPETS: &[Snippet] = &[
    Snippet {
        name: "Rust // packet parser",
        language: "rust",
        lines: &[
            "use std::net::Ipv4Addr;",
            "",
            "#[derive(Debug, Clone)]",
            "struct Packet {",
            "    source: Ipv4Addr,",
            "    destination: Ipv4Addr,",
            "    payload: Vec<u8>,",
            "}",
            "",
            "fn inspect(packet: &Packet) -> bool {",
            "    let suspicious = packet.payload.starts_with(b\"VIM\");",
            "    suspicious && packet.source.is_private()",
            "}",
        ],
    },
    Snippet {
        name: "Python // log scanner",
        language: "python",
        lines: &[
            "from pathlib import Path",
            "",
            "def scan_log(path: Path) -> list[str]:",
            "    signals = []",
            "    for line in path.read_text().splitlines():",
            "        if \"DENIED\" in line or \"ALERT\" in line:",
            "            signals.append(line.strip())",
            "    return signals",
            "",
            "print(scan_log(Path(\"/var/log/vimurai.log\")))",
        ],
    },
    Snippet {
        name: "JSON // access matrix",
        language: "json",
        lines: &[
            "{",
            "  \"agent\": \"kage\",",
            "  \"status\": \"online\",",
            "  \"ports\": [22, 80, 443],",
            "  \"permissions\": {",
            "    \"read\": true,",
            "    \"write\": false",
            "  }",
            "}",
        ],
    },
    Snippet {
        name: "LOG // night watch",
        language: "text",
        lines: &[
            "00:00:01 [BOOT] neural motion grid online",
            "00:00:03 [INFO] watcher kage connected",
            "00:01:17 [WARN] repeated lateral movement detected",
            "00:01:18 [HINT] consider using w or f{char}",
            "00:02:04 [OK] target acquired in two actions",
            "00:03:33 [SYNC] review schedule committed",
        ],
    },
];

#[must_use]
pub fn snippet(index: usize) -> &'static Snippet {
    &SNIPPETS[index % SNIPPETS.len()]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_snippet_has_content_and_wraps() {
        assert!(SNIPPETS.iter().all(|item| !item.lines.is_empty()));
        assert_eq!(snippet(SNIPPETS.len()), &SNIPPETS[0]);
    }
}
