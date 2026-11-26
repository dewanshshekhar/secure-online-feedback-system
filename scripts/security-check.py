"""Small, reviewable SAST policy checks; complements compiler lints and manual review."""
from pathlib import Path
import re
import sys

root = Path(__file__).resolve().parents[1]
rules = {
    "raw HTML rendering": r"\b(?:inner_html|dangerously_set_inner_html)\b",
    "unsafe Rust block": r"\bunsafe\s*\{",
    "browser persistent storage": r"\b(?:localStorage|sessionStorage|local_storage|session_storage)\b",
}
findings = []
for directory in (root / "backend/src", root / "frontend/src"):
    for path in directory.rglob("*.rs"):
        for number, line in enumerate(path.read_text(encoding="utf-8").splitlines(), 1):
            for name, pattern in rules.items():
                if re.search(pattern, line):
                    findings.append(f"{path.relative_to(root)}:{number}: prohibited {name}")
print("\n".join(findings) if findings else "Source security policy checks passed.")
sys.exit(bool(findings))
