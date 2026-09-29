from pathlib import Path

ROOT = Path(r"E:\Tokn\V0-CodexTkn-Consume\tool")

REPLACEMENTS = {
    "pb-observe": "tokn-observe",
    "pb-domain": "tokn-domain",
    "pb-platform": "tokn-platform",
    "pb-ingest": "tokn-ingest",
    "pb-codex": "tokn-codex",
    "pb-analysis": "tokn-analysis",
    "pb-storage": "tokn-storage",
    "pb-report": "tokn-report",
    "pb_domain": "tokn_domain",
    "pb_platform": "tokn_platform",
    "pb_ingest": "tokn_ingest",
    "pb_codex": "tokn_codex",
    "pb_analysis": "tokn_analysis",
    "pb_storage": "tokn_storage",
    "pb_report": "tokn_report",
    "ProjectBrain": "Tokn",
    "PROJECTBRAIN_CODEX_BIN": "TOKN_CODEX_BIN",
    "PBEVENT": "TOKNEVENT",
    "PBTRACE": "TOKNTRACE",
    "pbevent/0.1": "toknevent/0.1",
    "pbtrace/0.1": "tokntrace/0.1",
}

TEXT_EXTS = {
    ".rs", ".toml", ".md", ".ps1", ".cmd", ".yml", ".yaml",
    ".sql", ".json", ".jsonl", ".txt"
}

for path in ROOT.rglob("*"):
    if not path.is_file() or "target" in path.parts:
        continue
    if path.suffix.lower() not in TEXT_EXTS and path.name != "Cargo.lock":
        continue
    try:
        text = path.read_text(encoding="utf-8")
    except UnicodeDecodeError:
        continue
    new = text
    for old, replacement in REPLACEMENTS.items():
        new = new.replace(old, replacement)
    if new != text:
        path.write_text(new, encoding="utf-8", newline="\n")

RENAMES = [
    (ROOT / "apps" / "pb-observe", ROOT / "apps" / "tokn-observe"),
    (ROOT / "crates" / "pb-domain", ROOT / "crates" / "tokn-domain"),
    (ROOT / "crates" / "pb-platform", ROOT / "crates" / "tokn-platform"),
    (ROOT / "crates" / "pb-ingest", ROOT / "crates" / "tokn-ingest"),
    (ROOT / "crates" / "pb-codex", ROOT / "crates" / "tokn-codex"),
    (ROOT / "crates" / "pb-analysis", ROOT / "crates" / "tokn-analysis"),
    (ROOT / "crates" / "pb-storage", ROOT / "crates" / "tokn-storage"),
    (ROOT / "crates" / "pb-report", ROOT / "crates" / "tokn-report"),
]

for source, destination in RENAMES:
    if source.exists() and not destination.exists():
        source.rename(destination)

print("Tokn rename complete")
