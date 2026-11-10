#!/usr/bin/env python3
"""Count CSS properties and selector features in EPUB fixtures.

Reads every .epub under the given directories. Writes a JSON dump and a
markdown table. A reviewer reruns this script. It does not render pages.
"""

import argparse
import json
import re
import zipfile
from collections import Counter
from pathlib import Path

CHROME_SELECTOR = re.compile(
    r"titlepage|colophon|imprint|cover|endnote|toc\b|nav\b|loi\b|loa\b|landmark",
    re.I,
)
CHAPTER_NAME = re.compile(
    r"chapter|act-|scene|text/|bodymatter|\.xhtml$|\.html$|\.htm$",
    re.I,
)
SKIP_NAME = re.compile(r"nav\.|toc\.|colophon|titlepage|imprint|cover|license", re.I)
VALUE_PROPS = {
    "text-align",
    "font-style",
    "font-weight",
    "font-variant",
    "font-variant-numeric",
    "hyphens",
    "-webkit-hyphens",
    "text-indent",
    "line-height",
    "font-size",
    "text-transform",
    "white-space",
    "display",
    "position",
    "vertical-align",
    "hanging-punctuation",
    "line-break",
    "text-autospace",
    "word-break",
    "text-wrap",
    "list-style-type",
    "float",
    "clear",
    "writing-mode",
    "text-orientation",
    "direction",
}


def strip_comments(css: str) -> str:
    out = []
    i = 0
    n = len(css)
    while i < n:
        if css.startswith("/*", i):
            j = css.find("*/", i + 2)
            if j < 0:
                break
            i = j + 2
            continue
        out.append(css[i])
        i += 1
    return "".join(out)


def skip_ws(text, i):
    n = len(text)
    while i < n and text[i].isspace():
        i += 1
    return i


def read_string(text, i):
    quote = text[i]
    i += 1
    n = len(text)
    while i < n:
        if text[i] == "\\":
            i += 2
            continue
        if text[i] == quote:
            return i + 1
        i += 1
    return n


def read_until_block_or_semi(text, i):
    n = len(text)
    start = i
    depth = 0
    while i < n:
        c = text[i]
        if c in "\"'":
            i = read_string(text, i)
            continue
        if c == "(":
            depth += 1
        elif c == ")" and depth:
            depth -= 1
        elif c == "{" and depth == 0:
            return text[start:i], i, "{"
        elif c == ";" and depth == 0:
            return text[start:i], i, ";"
        i += 1
    return text[start:], n, ""


def read_block(text, i):
    """i points at '{'. Returns inner text and index after '}'."""
    assert text[i] == "{"
    i += 1
    n = len(text)
    start = i
    depth = 1
    while i < n:
        c = text[i]
        if c in "\"'":
            i = read_string(text, i)
            continue
        if c == "{":
            depth += 1
        elif c == "}":
            depth -= 1
            if depth == 0:
                return text[start:i], i + 1
        i += 1
    return text[start:], n


def split_top_commas(selector):
    parts = []
    buf = []
    depth = 0
    i = 0
    n = len(selector)
    while i < n:
        c = selector[i]
        if c in "\"'":
            j = read_string(selector, i)
            buf.append(selector[i:j])
            i = j
            continue
        if c in "([":
            depth += 1
        elif c in ")]" and depth:
            depth -= 1
        elif c == "," and depth == 0:
            parts.append("".join(buf))
            buf = []
            i += 1
            continue
        buf.append(c)
        i += 1
    parts.append("".join(buf))
    return parts


def selector_features(selector):
    counts = Counter()
    for part in split_top_commas(selector):
        part = part.strip()
        if not part:
            continue
        counts["selector"] += 1
        i = 0
        n = len(part)
        saw_compound = False
        while i < n:
            c = part[i]
            if c.isspace():
                j = skip_ws(part, i)
                nxt = part[j] if j < n else ""
                if nxt and nxt not in ">+~":
                    counts["descendant"] += 1
                i = j
                continue
            if c == ">":
                counts["child"] += 1
                i += 1
                continue
            if c == "+":
                counts["adjacent-sibling"] += 1
                i += 1
                continue
            if c == "~":
                counts["general-sibling"] += 1
                i += 1
                continue
            if c == "*":
                counts["universal"] += 1
                saw_compound = True
                i += 1
                continue
            if c == ".":
                counts["class"] += 1
                saw_compound = True
                i += 1
                while i < n and part[i] not in " \t\n>+~.:#[,":
                    i += 1
                continue
            if c == "#":
                counts["id"] += 1
                saw_compound = True
                i += 1
                while i < n and part[i] not in " \t\n>+~.:#[,":
                    i += 1
                continue
            if c == "[":
                counts["attribute"] += 1
                saw_compound = True
                end = part.find("]", i)
                body = part[i + 1 : end if end > 0 else n]
                if "|=" in body or "~=" in body or "^=" in body or "$=" in body or "*=" in body:
                    counts["attribute-operator"] += 1
                if "|" in body.split("=")[0]:
                    counts["attribute-namespace"] += 1
                i = end + 1 if end > 0 else n
                continue
            if c == ":":
                if part.startswith("::", i):
                    counts["pseudo-element"] += 1
                    name, i = read_pseudo_name(part, i + 2)
                    counts[f"pseudo-element:{name}"] += 1
                else:
                    name, i = read_pseudo_name(part, i + 1)
                    counts["pseudo-class"] += 1
                    counts[f"pseudo-class:{name}"] += 1
                    if name in {"is", "where", "not", "has", "nth-child", "nth-of-type", "nth-last-child"}:
                        counts[f"pseudo-functional:{name}"] += 1
                saw_compound = True
                continue
            if c == "|":
                counts["namespace-type"] += 1
                i += 1
                continue
            if c.isalpha() or c == "_" or c == "\\":
                counts["type"] += 1
                saw_compound = True
                while i < n and part[i] not in " \t\n>+~.:#[,|":
                    i += 1
                continue
            i += 1
        if saw_compound:
            counts["compound"] += 1
    if len([part for part in split_top_commas(selector) if part.strip()]) > 1:
        counts["selector-list"] += 1
    return counts


def read_pseudo_name(part, i):
    n = len(part)
    start = i
    while i < n and (part[i].isalnum() or part[i] == "-"):
        i += 1
    name = part[start:i].lower() or "(anon)"
    if i < n and part[i] == "(":
        depth = 1
        i += 1
        while i < n and depth:
            if part[i] in "\"'":
                i = read_string(part, i)
                continue
            if part[i] == "(":
                depth += 1
            elif part[i] == ")":
                depth -= 1
            i += 1
    return name, i


def declarations(block):
    """Yield (property, value, important) from a declaration block."""
    i = 0
    n = len(block)
    while i < n:
        i = skip_ws(block, i)
        if i >= n:
            break
        if block[i] == ";":
            i += 1
            continue
        start = i
        while i < n and block[i] not in ":;{}":
            if block[i] in "\"'":
                i = read_string(block, i)
                continue
            i += 1
        prop = block[start:i].strip().lower()
        if i >= n or block[i] != ":" or not prop or prop.startswith("@"):
            if i < n and block[i] == "{":
                _, i = read_block(block, i)
            else:
                i += 1
            continue
        i += 1
        vstart = i
        depth = 0
        while i < n:
            c = block[i]
            if c in "\"'":
                i = read_string(block, i)
                continue
            if c == "(":
                depth += 1
            elif c == ")" and depth:
                depth -= 1
            elif c in ";}" and depth == 0:
                break
            i += 1
        raw = block[vstart:i].strip()
        important = bool(re.search(r"!important\s*$", raw, re.I))
        value = re.sub(r"\s*!important\s*$", "", raw, flags=re.I).strip()
        value = re.sub(r"\s+", " ", value)
        if prop:
            yield prop, value, important
        if i < n and block[i] == ";":
            i += 1


def walk_sheet(css, sink, at_stack=()):
    text = strip_comments(css)
    i = 0
    n = len(text)
    while i < n:
        i = skip_ws(text, i)
        if i >= n:
            break
        if text[i] == "@":
            prelude, j, kind = read_until_block_or_semi(text, i)
            match = re.match(r"@[A-Za-z-]+", prelude.strip())
            name = match.group(0).lower() if match else prelude.strip().split(None, 1)[0].lower()
            sink["at"][name] += 1
            if kind == "{":
                inner, i = read_block(text, j)
                nested = name in {
                    "@media",
                    "@supports",
                    "@layer",
                    "@container",
                    "@document",
                    "@scope",
                }
                keyframes = name in {"@keyframes", "@-webkit-keyframes"}
                if nested:
                    walk_sheet(inner, sink, at_stack + (name,))
                elif keyframes:
                    sink["at"]["@keyframes-block"] += 1
                else:
                    for prop, value, important in declarations(inner):
                        record(sink, prop, value, important, prelude.strip(), at_stack + (name,))
            else:
                i = j + 1 if kind == ";" else j
            continue
        prelude, j, kind = read_until_block_or_semi(text, i)
        if kind != "{":
            i = j + 1 if kind else n
            continue
        inner, i = read_block(text, j)
        selector = re.sub(r"\s+", " ", prelude).strip()
        if not selector:
            continue
        chrome = bool(CHROME_SELECTOR.search(selector))
        feats = selector_features(selector)
        for key, count in feats.items():
            sink["features"][key] += count
            if not chrome:
                sink["features_body"][key] += count
        for prop, value, important in declarations(inner):
            record(sink, prop, value, important, selector, at_stack, chrome=chrome)


def record(sink, prop, value, important, selector, at_stack, chrome=False):
    sink["props"][prop] += 1
    if important:
        sink["important"][prop] += 1
    if at_stack:
        sink["props_in_at"][prop] += 1
    if not chrome:
        sink["props_body"][prop] += 1
    if prop in VALUE_PROPS:
        short = value if len(value) <= 80 else value[:77] + "..."
        sink["values"][f"{prop} = {short}"] += 1
        if not chrome:
            sink["values_body"][f"{prop} = {short}"] += 1


def new_sink():
    return {
        "props": Counter(),
        "props_body": Counter(),
        "props_in_at": Counter(),
        "important": Counter(),
        "features": Counter(),
        "features_body": Counter(),
        "values": Counter(),
        "values_body": Counter(),
        "at": Counter(),
        "inline": Counter(),
        "elements": Counter(),
        "elements_chapter": Counter(),
    }


def html_bits(html, sink, chapter_file):
    for style in re.findall(r"(?is)<style[^>]*>(.*?)</style>", html):
        walk_sheet(style, sink)
    for inline in re.findall(r'(?is)\sstyle\s*=\s*([\'"])(.*?)\1', html):
        for prop, value, important in declarations(inline[1]):
            sink["inline"][prop] += 1
            record(sink, prop, value, important, "<style-attr>", (), chrome=False)
    for tag in re.findall(r"(?is)<([a-zA-Z][\w:-]*)\b", html):
        name = tag.lower()
        sink["elements"][name] += 1
        if chapter_file:
            sink["elements_chapter"][name] += 1


def survey_epub(path: Path):
    sink = new_sink()
    files = []
    with zipfile.ZipFile(path) as book:
        for info in book.infolist():
            name = info.filename
            lower = name.lower()
            if lower.endswith(".css"):
                text = book.read(name).decode("utf-8", "replace")
                walk_sheet(text, sink)
                files.append(name)
            elif lower.endswith((".xhtml", ".html", ".htm", ".xml")):
                text = book.read(name).decode("utf-8", "replace")
                chapter = bool(CHAPTER_NAME.search(name)) and not SKIP_NAME.search(name)
                html_bits(text, sink, chapter)
                files.append(name)
    return sink, files


def merge(into, src):
    for key, counter in src.items():
        into[key].update(counter)


def table(counter, limit=None):
    rows = counter.most_common(limit)
    lines = ["| Name | Count |", "|---|---:|"]
    for name, count in rows:
        safe = name.replace("|", "\\|")
        lines.append(f"| `{safe}` | {count} |")
    if not rows:
        lines.append("| (none) | 0 |")
    return "\n".join(lines)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("dirs", nargs="+", type=Path)
    parser.add_argument("--json", type=Path, required=True)
    parser.add_argument("--md", type=Path, required=True)
    args = parser.parse_args()
    books = []
    total = new_sink()
    for directory in args.dirs:
        for path in sorted(directory.rglob("*.epub")):
            sink, files = survey_epub(path)
            merge(total, sink)
            books.append(
                {
                    "path": str(path),
                    "name": path.name,
                    "files": len(files),
                    "declarations": sum(sink["props"].values()),
                    "properties": len(sink["props"]),
                    "top_props": sink["props"].most_common(12),
                    "top_elements": sink["elements"].most_common(15),
                }
            )
    payload = {
        "books": books,
        "props": total["props"].most_common(),
        "props_body": total["props_body"].most_common(),
        "props_in_at": total["props_in_at"].most_common(),
        "important": total["important"].most_common(),
        "features": total["features"].most_common(),
        "features_body": total["features_body"].most_common(),
        "values": total["values"].most_common(80),
        "values_body": total["values_body"].most_common(80),
        "at": total["at"].most_common(),
        "inline": total["inline"].most_common(),
        "elements": total["elements"].most_common(),
        "elements_chapter": total["elements_chapter"].most_common(),
    }
    args.json.parent.mkdir(parents=True, exist_ok=True)
    args.json.write_text(json.dumps(payload, indent=2) + "\n")
    md = []
    md.append("# CSS survey")
    md.append("")
    md.append(
        "Counts are declaration occurrences, not distinct rules. "
        "`props_body` drops selectors that mention title page, colophon, imprint, cover, endnotes, or nav. "
        "The script is `crates/epub-native-spike/survey_css.py`."
    )
    md.append("")
    md.append("## Books")
    md.append("")
    md.append("| Book | Files | Declarations | Distinct properties |")
    md.append("|---|---:|---:|---:|")
    for book in books:
        md.append(
            f"| `{book['name']}` | {book['files']} | {book['declarations']} | {book['properties']} |"
        )
    md.append("")
    md.append("## Properties, all stylesheets")
    md.append("")
    md.append(table(total["props"]))
    md.append("")
    md.append("## Properties outside chrome selectors")
    md.append("")
    md.append(table(total["props_body"]))
    md.append("")
    md.append("## Selector features")
    md.append("")
    md.append(table(total["features"]))
    md.append("")
    md.append("## At-rules")
    md.append("")
    md.append(table(total["at"]))
    md.append("")
    md.append("## Inline style properties")
    md.append("")
    md.append(table(total["inline"]))
    md.append("")
    md.append("## Chapter-like elements")
    md.append("")
    md.append(table(total["elements_chapter"]))
    md.append("")
    md.append("## Values that change layout")
    md.append("")
    md.append(table(Counter(dict(total["values_body"].most_common(60)))))
    md.append("")
    args.md.write_text("\n".join(md))
    print(f"books {len(books)} properties {len(total['props'])} declarations {sum(total['props'].values())}")
    print(args.md)
    print(args.json)


if __name__ == "__main__":
    main()
