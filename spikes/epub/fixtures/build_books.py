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


def plain_from(epub, needle):
    if not epub.exists():
        return ""
    with zipfile.ZipFile(epub) as book:
        for name in book.namelist():
            if not name.lower().endswith((".xhtml", ".html", ".htm")):
                continue
            text = book.read(name).decode("utf-8", "replace")
            if needle not in text:
                continue
            body = re.sub(r"(?is)<(script|style)[^>]*>.*?</\1>", " ", text)
            body = re.sub(r"(?is)<[^>]+>", "\n", body)
            body = html.unescape(body)
            body = re.sub(r"[ \t]+", " ", body)
            body = re.sub(r"\n+", "\n", body).strip()
            if len(body) > 1500:
                return body[:6000]
    return ""


def paragraphs(text, fallback):
    chunks = []
    for block in re.split(r"\n+", text):
        block = " ".join(block.split())
        if "Gutenberg" in block or "LICENSE" in block:
            continue
        block = block.replace("\ue4c2", "").replace("\ue000", "")
        if len(block) >= 12:
            chunks.append(block)
    if not chunks:
        chunks = [fallback]
    filled = []
    while sum(len(chunk) for chunk in filled) < 2200:
        filled.extend(chunks[:8])
    return filled


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


def p_tags(chunks):
    return "\n".join(f"<p>{html.escape(chunk)}</p>" for chunk in chunks)


def build():
    pride = paragraphs(
        plain_from(PRIDE, "truth universally acknowledged"),
        "It is a truth universally acknowledged, that a single man in possession of a good fortune, must be in want of a wife. "
        "However little known the feelings or views of such a man may be on his first entering a neighbourhood, "
        "this truth is so well fixed in the minds of the surrounding families, that he is considered the rightful property of some one or other of their daughters.",
    )
    chinese = paragraphs(
        plain_from(ZHAOHUA, "范"),
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
"""
    books = {
        "plain-en.epub": (
            "en",
            "Plain English",
            p_tags(pride),
            "",
            None,
            [],
        ),
        "plain-zh.epub": (
            "zh-CN",
            "Plain Chinese",
            p_tags(chinese),
            "",
            None,
            [],
        ),
        "styled.epub": (
            "en",
            "Styled Literata",
            p_tags(pride),
            '<link rel="stylesheet" href="style.css"/>',
            None,
            [("style", "style.css", "text/css"), ("font", "Literata.ttf", "font/ttf")],
        ),
        "fixed.epub": (
            "en",
            "Fixed Square",
            p_tags(pride),
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
