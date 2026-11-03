#!/usr/bin/env python3
"""Build the small EPUBs the reading-view checks open."""

import html
import re
import zipfile
from pathlib import Path

HERE = Path(__file__).resolve().parent
FONT = HERE / "Literata.ttf"
OUT = Path("/tmp/superreader-fixtures")
PRIDE = OUT / "pride-se.epub"
ZHAOHUA = OUT / "zhaohua.epub"


def blocks_from(epub, needle, fallback):
    blocks = []
    if epub.exists():
        with zipfile.ZipFile(epub) as book:
            for name in book.namelist():
                if not name.lower().endswith((".xhtml", ".html", ".htm")):
                    continue
                text = book.read(name).decode("utf-8", "replace")
                if needle not in text:
                    continue
                body = re.search(r"(?is)<body[^>]*>(.*)</body>", text)
                inner = body.group(1) if body else text
                for match in re.finditer(r"(?is)<(h[1-6]|p)\b[^>]*>(.*?)</\1>", inner):
                    tag = match.group(1).lower()
                    piece = re.sub(r"(?is)<[^>]+>", "", match.group(2))
                    piece = html.unescape(piece)
                    piece = " ".join(piece.split()).replace("\ue4c2", "").replace("\ue000", "")
                    if not piece or "Gutenberg" in piece or "LICENSE" in piece:
                        continue
                    kind = tag if tag.startswith("h") else "p"
                    block = (kind, piece)
                    if blocks and blocks[-1] == block:
                        continue
                    blocks.append(block)
                if sum(len(piece) for _, piece in blocks) > 1500:
                    break
    if not blocks:
        blocks = [("p", fallback)]
    return blocks


def render(blocks):
    return "\n".join(f"<{tag}>{html.escape(text)}</{tag}>" for tag, text in blocks)


def xhtml(lang, title, body, extra_head=""):
    return f"""<?xml version="1.0" encoding="utf-8"?>
<html xmlns="http://www.w3.org/1999/xhtml" xml:lang="{lang}" lang="{lang}">
<head>
<title>{html.escape(title)}</title>
{extra_head}
</head>
<body>
{body}
</body>
</html>
"""


def opf(lang, title, items, viewport=None):
    metas = ""
    if viewport:
        width, height = viewport
        metas = (
            '<meta property="rendition:layout">pre-paginated</meta>'
            f'<meta property="rendition:viewport">width={width}, height={height}</meta>'
        )
    manifest = "\n".join(
        f'<item id="{item_id}" href="{href}" media-type="{mime}"/>'
        for item_id, href, mime in items
    )
    spine = '<itemref idref="chapter"/>'
    return f"""<?xml version="1.0" encoding="utf-8"?>
<package xmlns="http://www.idpf.org/2007/opf" version="3.0" unique-identifier="id">
<metadata xmlns:dc="http://purl.org/dc/elements/1.1/">
<dc:identifier id="id">superreader-fixture</dc:identifier>
<dc:title>{html.escape(title)}</dc:title>
<dc:language>{lang}</dc:language>
{metas}
</metadata>
<manifest>
{manifest}
</manifest>
<spine>{spine}</spine>
</package>
"""


def write_epub(path, files):
    path.parent.mkdir(parents=True, exist_ok=True)
    with zipfile.ZipFile(path, "w") as book:
        info = zipfile.ZipInfo("mimetype")
        info.compress_type = zipfile.ZIP_STORED
        book.writestr(info, "application/epub+zip")
        for name, data in files.items():
            book.writestr(name, data)


def build():
    pride = blocks_from(
        PRIDE,
        "truth universally acknowledged",
        "It is a truth universally acknowledged, that a single man in possession of a good fortune, must be in want of a wife.",
    )
    chinese = blocks_from(
        ZHAOHUA,
        "范",
        "在東京的客店裏，我們大抵一起來就看報。學生所看的多是朝日新聞和讀賣新聞。專愛看一點報上的雜感。",
    )
    container = """<?xml version="1.0"?>
<container version="1.0" xmlns="urn:oasis:names:tc:opendocument:xmlns:container">
<rootfiles><rootfile full-path="OEBPS/content.opf" media-type="application/oebps-package+xml"/></rootfiles>
</container>
"""
    styled_css = """
@font-face { font-family: "Literata"; src: url("Literata.ttf") format("truetype"); }
p { font-family: "Literata", serif; font-size: 19px; line-height: 1.65; text-indent: 1.5em; text-align: left; margin: 0 0 0.8em 0; }
h1, h2, h3 { font-family: "Literata", serif; font-size: 19px; text-align: center; text-indent: 0; font-weight: 600; margin: 0 0 1em 0; }
"""
    books = {
        "plain-en.epub": (
            "en",
            "Plain English",
            render(pride),
            "",
            None,
            [],
        ),
        "plain-zh.epub": (
            "zh-CN",
            "Plain Chinese",
            render(chinese),
            "",
            None,
            [],
        ),
        "styled.epub": (
            "en",
            "Styled Literata",
            render(pride),
            '<link rel="stylesheet" href="style.css"/>',
            None,
            [("style", "style.css", "text/css"), ("font", "Literata.ttf", "font/ttf")],
        ),
        "fixed.epub": (
            "en",
            "Fixed Square",
            render(pride),
            '<meta name="viewport" content="width=1000, height=1000"/>',
            (1000, 1000),
            [],
        ),
    }
    for name, (lang, title, body, head, viewport, extra) in books.items():
        items = [("chapter", "chapter.xhtml", "application/xhtml+xml"), *extra]
        files = {
            "META-INF/container.xml": container.encode(),
            "OEBPS/content.opf": opf(lang, title, items, viewport).encode(),
            "OEBPS/chapter.xhtml": xhtml(lang, title, body, head).encode(),
        }
        if name == "styled.epub":
            files["OEBPS/style.css"] = styled_css.encode()
            files["OEBPS/Literata.ttf"] = FONT.read_bytes()
        write_epub(OUT / name, files)
        print(OUT / name)


if __name__ == "__main__":
    build()
