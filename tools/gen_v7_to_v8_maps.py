"""Generate human-readable Rust v7→v8 mapping tables from the upstream C++ source.

This is intentionally NOT used at build time.

The Rust port vendors the mapping entries into source control so the crate builds
without requiring the upstream C++ repo.

Usage (from repo root):
    python tools/gen_v7_to_v8_maps.py

You can also override paths:
    python tools/gen_v7_to_v8_maps.py --cpp path/to/PatchVersion7to8_unorderedmaps.cpp \
        --out crates/gb-save-polished/src/migrations/v7_to_v8_maps_data.rs
"""

from __future__ import annotations

import argparse
import pathlib
import re
from dataclasses import dataclass
from typing import Iterable, List, Optional


REPO_ROOT = pathlib.Path(__file__).resolve().parents[1]
DEFAULT_CPP = (
    REPO_ROOT
    / "third_party"
    / "polished-save-patcher"
    / "src"
    / "patching"
    / "PatchVersion7to8_unorderedmaps.cpp"
)
DEFAULT_OUT = (
    REPO_ROOT
    / "crates"
    / "gb-save-polished"
    / "src"
    / "migrations"
    / "v7_to_v8_maps_data.rs"
)


@dataclass(frozen=True)
class MapSpec:
    rust_name: str
    cpp_func: str
    kind: str


SPECS: List[MapSpec] = [
    MapSpec("KEY_ITEM", "mapV7KeyItemToV8", "u8_u8"),
    MapSpec("ITEM", "mapV7ItemToV8", "u8_u8"),
    MapSpec("EVENT_FLAG", "mapV7EventFlagToV8", "u16_u16"),
    MapSpec("LANDMARK", "mapV7LandmarkToV8", "u8_u8"),
    MapSpec("SPAWN", "mapV7SpawnToV8", "u8_u8"),
    MapSpec("PKMN", "mapV7PkmnToV8", "u8_u16"),
    MapSpec("MAP_GROUP_NUMBER", "mapv7toV8", "tuple_u8_u8_to_u8_u8"),
    MapSpec(
        "SPECIES_FORM_TO_EXTSPECIES",
        "mapV7SpeciesFormToV8Extspecies",
        "tuple_u8_u8_to_u16",
    ),
    MapSpec("MAGIKARP_FORM", "mapV7MagikarpFormToV8", "u8_u8"),
    MapSpec("THEME", "mapV7ThemeToV8", "u8_u8"),
    MapSpec("CHAR", "mapV7CharToV8", "u8_u8"),
]


def _find_function_block(cpp: str, func_name: str) -> str:
    idx = cpp.find(f"{func_name}(")
    if idx < 0:
        raise ValueError(f"Function {func_name} not found")

    tail = cpp[idx:]
    open_idx = tail.find("{")
    if open_idx < 0:
        raise ValueError(f"No '{{' found after {func_name}")

    depth = 0
    for rel, ch in enumerate(tail[open_idx:], start=open_idx):
        if ch == "{":
            depth += 1
        elif ch == "}":
            depth -= 1
            if depth == 0:
                return tail[: rel + 1]

    raise ValueError(f"Unterminated function {func_name}")


def _extract_initializer_lines(block: str) -> List[str]:
    """Extract the initializer-list body lines, preserving comments/spacing."""

    lines: List[str] = []
    in_init = False

    for raw in block.splitlines():
        stripped = raw.strip("\n")
        if not in_init:
            if "=" in stripped and "{" in stripped:
                in_init = True
            continue

        if stripped.strip().startswith("};"):
            break

        line = stripped.strip()
        if not line:
            lines.append("")
            continue

        lines.append(line)

    while lines and lines[-1] == "":
        lines.pop()

    return lines


_PAIR_RE = re.compile(
    r"^\{\s*(?P<a>[^,}]+)\s*,\s*(?P<b>[^}]+)\s*\}\s*,?\s*(?P<comment>//.*)?$"
)
_TUPLE_TO_TUPLE_RE = re.compile(
    r"^\{\{\s*(?P<a>[^,}]+)\s*,\s*(?P<b>[^}]+)\s*\}\s*,\s*\{\s*(?P<c>[^,}]+)\s*,\s*(?P<d>[^}]+)\s*\}\}\s*,?\s*(?P<comment>//.*)?$"
)
_TUPLE_TO_VALUE_RE = re.compile(
    r"^\{\{\s*(?P<a>[^,}]+)\s*,\s*(?P<b>[^}]+)\s*\}\s*,\s*(?P<c>[^}]+)\}\s*,?\s*(?P<comment>//.*)?$"
)


def _emit_array_header(name: str, ty: str) -> List[str]:
    return [
        "#[rustfmt::skip]",
        f"pub const {name}_ENTRIES: {ty} = &[",
    ]


def _emit_array_footer() -> List[str]:
    return [
        "];",
        "",
    ]


def _emit_line(line: str, kind: str) -> str:
    if not line:
        return ""

    if line.startswith("//"):
        return line

    if kind in {"u8_u8", "u16_u16", "u8_u16"}:
        m = _PAIR_RE.match(line)
        if not m:
            raise ValueError(f"Unparsed pair line: {line}")
        a = m.group("a").strip()
        b = m.group("b").strip()
        comment = (m.group("comment") or "").rstrip()
        out = f"    ({a}, {b}),"
        return f"{out}  {comment}".rstrip()

    if kind == "tuple_u8_u8_to_u8_u8":
        m = _TUPLE_TO_TUPLE_RE.match(line)
        if not m:
            raise ValueError(f"Unparsed tuple->tuple line: {line}")
        a = m.group("a").strip()
        b = m.group("b").strip()
        c = m.group("c").strip()
        d = m.group("d").strip()
        comment = (m.group("comment") or "").rstrip()
        out = f"    (({a}, {b}), ({c}, {d})),"
        return f"{out}  {comment}".rstrip()

    if kind == "tuple_u8_u8_to_u16":
        m = _TUPLE_TO_VALUE_RE.match(line)
        if not m:
            raise ValueError(f"Unparsed tuple->value line: {line}")
        a = m.group("a").strip()
        b = m.group("b").strip()
        c = m.group("c").strip()
        comment = (m.group("comment") or "").rstrip()
        out = f"    (({a}, {b}), {c}),"
        return f"{out}  {comment}".rstrip()

    raise ValueError(f"Unknown kind: {kind}")


def generate(cpp_text: str) -> str:
    out: List[str] = []

    out.extend(
        [
            "// This file is generated by tools/gen_v7_to_v8_maps.py.",
            "//",
            "// It is vendored into the repo to preserve upstream comments/formatting and",
            "// to keep builds independent from the upstream C++ code.",
            "",
        ]
    )

    for spec in SPECS:
        block = _find_function_block(cpp_text, spec.cpp_func)
        lines = _extract_initializer_lines(block)

        if spec.kind == "u8_u8":
            out.extend(_emit_array_header(spec.rust_name, "&[(u8, u8)]"))
        elif spec.kind == "u16_u16":
            out.extend(_emit_array_header(spec.rust_name, "&[(u16, u16)]"))
        elif spec.kind == "u8_u16":
            out.extend(_emit_array_header(spec.rust_name, "&[(u8, u16)]"))
        elif spec.kind == "tuple_u8_u8_to_u8_u8":
            out.extend(
                _emit_array_header(spec.rust_name, "&[((u8, u8), (u8, u8))]")
            )
        elif spec.kind == "tuple_u8_u8_to_u16":
            out.extend(_emit_array_header(spec.rust_name, "&[((u8, u8), u16)]"))
        else:
            raise ValueError(f"Unknown spec kind: {spec.kind}")

        for line in lines:
            out.append(_emit_line(line, spec.kind))

        out.extend(_emit_array_footer())

    return "\n".join(out)


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--cpp", type=pathlib.Path, default=DEFAULT_CPP)
    parser.add_argument("--out", type=pathlib.Path, default=DEFAULT_OUT)
    args = parser.parse_args()

    cpp_path: pathlib.Path = args.cpp
    out_path: pathlib.Path = args.out

    cpp_text = cpp_path.read_text(encoding="utf-8")
    rust = generate(cpp_text)

    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text(rust, encoding="utf-8", newline="\n")

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
