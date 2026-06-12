#!/usr/bin/env python3
"""Check that every checked-in example JSON configuration references the live schema."""
import json
import os
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SCHEMA_PATH = ROOT / "sqlpage/sqlpage.schema.json"
EXPECTED_SCHEMA = json.loads(SCHEMA_PATH.read_text())["$id"]
configs = []
for directory, subdirectories, files in os.walk(ROOT):
    subdirectories[:] = [name for name in subdirectories if name not in {".git", "node_modules", "target"}]
    if "sqlpage.json" in files:
        configs.append(Path(directory) / "sqlpage.json")

missing = []
for path in configs:
    configuration = json.loads(path.read_text())
    if configuration.get("$schema") != EXPECTED_SCHEMA:
        missing.append(path.relative_to(ROOT))
if missing:
    raise SystemExit("Configurations missing the live $schema reference:\n" + "\n".join(map(str, missing)))
print(f"Checked {len(configs)} example configurations")
