#!/usr/bin/env python3
"""Read-only staging SEO checks for the Aurora Sky /vi/about design proposal."""
from __future__ import annotations

import ipaddress
import json
import re
import secrets
import sys
import xml.etree.ElementTree as ET
from dataclasses import dataclass
from html.parser import HTMLParser
from urllib.error import HTTPError, URLError
from urllib.parse import urlsplit
from urllib.request import HTTPRedirectHandler, Request, build_opener

EXPECTED_TITLE = "A sky full of ideas, the vibe never stops — VionSky"
EXPECTED_DESCRIPTION = (
    "VionSky — nơi Creator, Người dùng và AI Agent gặp nhau để cùng làm việc. "
    "A sky full of ideas, the vibe never stops."
)
CANONICAL = "https://vionsky.com/vi/about"
EXPECTED_SITEMAP = "https://vionsky.com/sitemap.xml"
EXPECTED_H1 = "A sky full of ideas, the vibe never stops."
RAW_CONTENT_MARKERS = (
    "VionSky không làm ra sản phẩm AI.",
    "Con người giao việc, AI Agent thực thi",
    "Đánh giá minh bạch, không phán xét",
    "Agent bàn bạc cùng nhau để giải quyết task",
    "Ai cũng có chỗ đứng",
)
MAX_RESPONSE_BYTES = 2 * 1024 * 1024
TIMEOUT_SECONDS = 12


class NoRedirect(HTTPRedirectHandler):
    def redirect_request(self, request, response, code, message, headers, new_url):
        return None


@dataclass
class Response:
    status: int
    headers: object
    body: bytes


class Report:
    def __init__(self) -> None:
        self.passed = 0
        self.failed = 0

    def check(self, condition: bool, label: str, detail: str) -> None:
        state = "PASS" if condition else "FAIL"
        print(f"[{state}] {label}: {detail}")
        if condition:
            self.passed += 1
        else:
            self.failed += 1

    def finish(self) -> int:
        print(f"Summary: {self.passed} passed, {self.failed} failed.")
        print(
            "Evidence boundary: GET response checks only; raw initial HTTP HTML is inspected before hydration. "
            "This does not prove SSG, browser/SEO parity, or Worker CPU budget."
        )
        return 1 if self.failed else 0


class PageParser(HTMLParser):
    def __init__(self) -> None:
        super().__init__(convert_charrefs=True)
        self.titles: list[str] = []
        self._title: list[str] | None = None
        self.h1s: list[str] = []
        self._h1_stack: list[list[str]] = []
        self.metas: list[dict[str, str]] = []
        self.links: list[dict[str, str]] = []
        self.anchors: list[dict[str, object]] = []
        self._anchor_stack: list[int] = []
        self.jsonld: list[str] = []
        self._jsonld_index: int | None = None
        self.main_count = 0
        self.body_text: list[str] = []
        self._in_body = False
        self._suppressed_depth = 0

    def handle_starttag(self, tag: str, attrs: list[tuple[str, str | None]]) -> None:
        values = {key.lower(): value or "" for key, value in attrs}
        if tag == "body":
            self._in_body = True
        elif tag == "main":
            self.main_count += 1
        elif tag == "title":
            self._title = []
        elif tag == "meta":
            self.metas.append(values)
        elif tag == "link":
            self.links.append(values)
        elif tag == "h1":
            self._h1_stack.append([])
        elif tag == "a":
            self.anchors.append({"attrs": values, "text": []})
            self._anchor_stack.append(len(self.anchors) - 1)
        elif tag == "script":
            if values.get("type", "").split(";")[0].strip().lower() == "application/ld+json":
                self.jsonld.append("")
                self._jsonld_index = len(self.jsonld) - 1
            self._suppressed_depth += 1
        elif tag == "style":
            self._suppressed_depth += 1

    def handle_endtag(self, tag: str) -> None:
        if tag == "body":
            self._in_body = False
        elif tag == "title" and self._title is not None:
            self.titles.append(normalize_text("".join(self._title)))
            self._title = None
        elif tag == "h1" and self._h1_stack:
            self.h1s.append(normalize_text("".join(self._h1_stack.pop())))
        elif tag == "a" and self._anchor_stack:
            self._anchor_stack.pop()
        elif tag == "script":
            self._jsonld_index = None
            self._suppressed_depth = max(0, self._suppressed_depth - 1)
        elif tag == "style":
            self._suppressed_depth = max(0, self._suppressed_depth - 1)

    def handle_data(self, data: str) -> None:
        if self._title is not None:
            self._title.append(data)
        for current in self._h1_stack:
            current.append(data)
        for index in self._anchor_stack:
            text = self.anchors[index]["text"]
            assert isinstance(text, list)
            text.append(data)
        if self._jsonld_index is not None:
            self.jsonld[self._jsonld_index] += data
        if self._in_body and self._suppressed_depth == 0:
            self.body_text.append(data)


def normalize_text(value: str) -> str:
    return " ".join(value.split())


def stage_origin(value: str) -> tuple[str, str]:
    parsed = urlsplit(value)
    if parsed.scheme not in ("http", "https") or not parsed.hostname:
        raise ValueError("provide an absolute staging URL, e.g. https://staging.example")
    if parsed.username or parsed.password or parsed.query or parsed.fragment:
        raise ValueError("URL credentials, query strings, and fragments are not accepted")
    if parsed.path not in ("", "/"):
        raise ValueError("pass the staging origin only; checks use fixed paths")

    host = parsed.hostname.casefold().rstrip(".")
    try:
        ip = ipaddress.ip_address(host.strip("[]"))
        is_loopback = ip.is_loopback
    except ValueError:
        is_loopback = host == "localhost" or host.endswith(".localhost")

    if host in {"vionsky.com", "www.vionsky.com"}:
        raise ValueError("refusing the production VionSky host; use a staging host only")
    if not is_loopback and parsed.scheme != "https":
        raise ValueError("remote staging URLs must use HTTPS")
    marker = re.search(r"(?:^|[.-])(?:qa|test|testing|dev|stage|staging|preview)(?:[.-]|$)", host)
    if not is_loopback and not marker:
        raise ValueError(
            "host is not recognizably staging (expected a qa/test/dev/stage/staging/preview label)"
        )

    # Rebuild only the origin, retaining an explicit port and IPv6 brackets.
    netloc = parsed.netloc
    return f"{parsed.scheme}://{netloc}", host


def fetch(url: str) -> Response:
    request = Request(
        url,
        headers={
            "User-Agent": "Leptos-SEO-QA/1.0 (read-only staging checks)",
            "Accept": "text/html,application/xml,text/plain,*/*;q=0.5",
        },
        method="GET",
    )
    opener = build_opener(NoRedirect())
    try:
        response = opener.open(request, timeout=TIMEOUT_SECONDS)
    except HTTPError as error:
        response = error
    except (URLError, TimeoutError, OSError) as error:
        raise RuntimeError(str(error)) from error
    try:
        body = response.read(MAX_RESPONSE_BYTES + 1)
        if len(body) > MAX_RESPONSE_BYTES:
            raise RuntimeError(f"response exceeds {MAX_RESPONSE_BYTES} bytes")
        return Response(response.code, response.headers, body)
    finally:
        response.close()


def content_type(response: Response) -> str:
    return (response.headers.get("Content-Type") or "").split(";", 1)[0].strip().lower()


def get_checked(report: Report, origin: str, path: str, label: str) -> Response | None:
    try:
        response = fetch(origin + path)
    except RuntimeError as error:
        report.check(False, f"{label} HTTP", f"GET {path} failed: {error}")
        return None
    report.check(response.status == 200, f"{label} HTTP", f"GET {path} returned {response.status}; expected 200")
    return response


def jsonld_nodes(value: object):
    if isinstance(value, list):
        for item in value:
            yield from jsonld_nodes(item)
    elif isinstance(value, dict):
        yield value
        graph = value.get("@graph")
        if graph is not None:
            yield from jsonld_nodes(graph)


def type_names(value: object) -> list[str]:
    values = value if isinstance(value, list) else [value]
    return [str(item).rsplit("/", 1)[-1].rsplit("#", 1)[-1] for item in values]


def validate_jsonld(parser: PageParser) -> tuple[bool, str]:
    if not parser.jsonld:
        return False, "no application/ld+json block in the initial HTML"
    documents: list[object] = []
    try:
        documents = [json.loads(block) for block in parser.jsonld]
    except (json.JSONDecodeError, TypeError) as error:
        return False, f"invalid JSON-LD: {error}"
    about_pages = [
        node
        for document in documents
        for node in jsonld_nodes(document)
        if "AboutPage" in type_names(node.get("@type"))
    ]
    if len(about_pages) != 1:
        return False, f"expected exactly one AboutPage node, found {len(about_pages)}"
    node = about_pages[0]
    expected = {
        "name": EXPECTED_TITLE,
        "description": EXPECTED_DESCRIPTION,
        "url": CANONICAL,
        "inLanguage": "vi",
    }
    mismatches = [f"{key}={node.get(key)!r} (expected {value!r})" for key, value in expected.items() if node.get(key) != value]
    if mismatches:
        return False, "; ".join(mismatches)
    return True, "one valid AboutPage node has matching name, description, canonical URL, and inLanguage=vi"


def check_page(report: Report, response: Response, stage_host: str) -> None:
    html_type_ok = content_type(response) in {"text/html", "application/xhtml+xml"}
    report.check(html_type_ok, "raw HTML content type", f"Content-Type is {content_type(response) or '(missing)'}")
    try:
        html = response.body.decode("utf-8-sig")
    except UnicodeDecodeError as error:
        report.check(False, "raw HTML UTF-8", f"response is not UTF-8: {error}")
        return
    parser = PageParser()
    parser.feed(html)
    parser.close()

    report.check(parser.titles == [EXPECTED_TITLE], "title", f"expected exactly {EXPECTED_TITLE!r}; got {parser.titles!r}")
    descriptions = [meta.get("content", "") for meta in parser.metas if meta.get("name", "").casefold() == "description"]
    report.check(descriptions == [EXPECTED_DESCRIPTION], "description", f"expected one exact design description; found {descriptions!r}")

    canonicals = [link.get("href", "") for link in parser.links if "canonical" in link.get("rel", "").casefold().split()]
    report.check(canonicals == [CANONICAL], "canonical", f"expected exactly one {CANONICAL!r}; found {canonicals!r}")
    vietnamese = [
        link.get("href", "")
        for link in parser.links
        if "alternate" in link.get("rel", "").casefold().split()
        and link.get("hreflang", "").casefold() == "vi"
    ]
    report.check(CANONICAL in vietnamese, "hreflang", f"Vietnamese alternate must target {CANONICAL!r}; found {vietnamese!r}")

    json_ok, json_detail = validate_jsonld(parser)
    report.check(json_ok, "JSON-LD", json_detail)

    report.check(parser.main_count >= 1, "raw HTML main landmark", f"found {parser.main_count} <main> element(s) in the initial response")
    report.check(parser.h1s == [EXPECTED_H1], "raw HTML H1", f"expected one H1 {EXPECTED_H1!r}; found {parser.h1s!r}")
    primary_links = []
    for anchor in parser.anchors:
        attrs = anchor["attrs"]
        text = normalize_text("".join(anchor["text"]))  # type: ignore[arg-type]
        href = str(attrs.get("href", ""))  # type: ignore[union-attr]
        if text == "Khám phá War Room" and urlsplit(href).path == "/vi/channels":
            primary_links.append(href)
    report.check(bool(primary_links), "raw HTML primary link", f"expected 'Khám phá War Room' → /vi/channels; found {primary_links!r}")
    body_text = normalize_text("".join(parser.body_text))
    missing_copy = [marker for marker in RAW_CONTENT_MARKERS if marker not in body_text]
    report.check(not missing_copy, "raw HTML design copy", "all required body markers are present" if not missing_copy else f"missing {missing_copy!r}")

    x_robots = response.headers.get("X-Robots-Tag", "")
    header_noindex = any(token.strip().casefold() == "noindex" for token in re.split(r"[,;\s]+", x_robots) if token.strip())
    meta_noindex = any(
        meta.get("name", "").casefold() in {"robots", "googlebot"}
        and any(token.strip().casefold() == "noindex" for token in re.split(r"[,;\s]+", meta.get("content", "")) if token.strip())
        for meta in parser.metas
    )
    report.check(header_noindex or meta_noindex, "staging noindex", "noindex found in X-Robots-Tag or robots/googlebot meta")


def check_sitemap(report: Report, response: Response, stage_host: str) -> None:
    xml_type_ok = content_type(response) in {"application/xml", "text/xml"}
    report.check(xml_type_ok, "sitemap content type", f"Content-Type is {content_type(response) or '(missing)'}")
    if response.status != 200:
        return
    try:
        root = ET.fromstring(response.body)
        root_name = root.tag.rsplit("}", 1)[-1]
        locs = [element.text.strip() for element in root.iter() if element.tag.rsplit("}", 1)[-1] == "loc" and element.text and element.text.strip()]
        valid_root = root_name in {"urlset", "sitemapindex"}
        report.check(valid_root and bool(locs), "sitemap XML", f"valid {root_name!r} root with {len(locs)} loc value(s)")
        staging_locs = [loc for loc in locs if (urlsplit(loc).hostname or "").casefold().rstrip(".") == stage_host]
        report.check(not staging_locs, "sitemap staging isolation", "no staging-host URLs advertised" if not staging_locs else f"staging URLs found: {staging_locs!r}")
    except (ET.ParseError, UnicodeDecodeError) as error:
        report.check(False, "sitemap XML", f"invalid XML: {error}")


def check_text_resource(report: Report, response: Response, label: str) -> None:
    kind = content_type(response)
    type_ok = kind in {"text/plain", "text/markdown", "text/x-markdown", "application/markdown"}
    report.check(type_ok, f"{label} content type", f"Content-Type is {kind or '(missing)'}")
    if response.status != 200:
        return
    try:
        text = response.body.decode("utf-8-sig")
        report.check(bool(text.strip()), f"{label} body", "non-empty UTF-8 text is served")
    except UnicodeDecodeError as error:
        report.check(False, f"{label} body", f"response is not UTF-8: {error}")


def check_robots(report: Report, response: Response, stage_host: str) -> None:
    check_text_resource(report, response, "robots.txt")
    if response.status != 200:
        return
    try:
        text = response.body.decode("utf-8-sig")
    except UnicodeDecodeError:
        return
    directives: list[tuple[str, str]] = []
    for line in text.splitlines():
        directive = line.partition("#")[0].strip()
        if ":" not in directive:
            continue
        name, value = directive.split(":", 1)
        directives.append((name.strip().casefold(), value.strip()))
    user_agents = [value for name, value in directives if name == "user-agent" and value]
    sitemap_urls = [value for name, value in directives if name == "sitemap" and value]
    report.check(bool(user_agents), "robots.txt user-agent", "at least one non-empty User-agent directive is present")
    report.check(
        EXPECTED_SITEMAP in sitemap_urls,
        "robots.txt sitemap",
        f"expected {EXPECTED_SITEMAP!r} directive; found {sitemap_urls!r}",
    )
    disallow_all = any(name == "disallow" and value == "/" for name, value in directives)
    report.check(disallow_all, "staging robots disallow", f"{stage_host} must contain Disallow: /")


def check_llms(report: Report, response: Response) -> None:
    check_text_resource(report, response, "llms.txt")
    if response.status != 200:
        return
    try:
        text = response.body.decode("utf-8-sig")
    except UnicodeDecodeError:
        return
    first_line = next((line.strip() for line in text.splitlines() if line.strip()), "")
    report.check(
        first_line.startswith("# ") and bool(first_line[2:].strip()),
        "llms.txt heading",
        f"expected a non-empty Markdown H1 on the first non-empty line; found {first_line!r}",
    )


def run(origin: str, stage_host: str) -> int:
    report = Report()
    page = get_checked(report, origin, "/vi/about", "about page")
    if page is not None and page.status == 200:
        check_page(report, page, stage_host)

    sitemap = get_checked(report, origin, "/sitemap.xml", "sitemap.xml")
    if sitemap is not None:
        check_sitemap(report, sitemap, stage_host)

    robots = get_checked(report, origin, "/robots.txt", "robots.txt")
    if robots is not None:
        check_robots(report, robots, stage_host)

    llms = get_checked(report, origin, "/llms.txt", "llms.txt")
    if llms is not None:
        check_llms(report, llms)

    missing_path = f"/.well-known/seo-qa-missing-{secrets.token_hex(8)}"
    try:
        missing = fetch(origin + missing_path)
        report.check(missing.status == 404, "real 404", f"unknown GET route returned HTTP {missing.status}; expected 404 (not a styled 200)")
    except RuntimeError as error:
        report.check(False, "real 404", f"GET {missing_path} failed: {error}")

    return report.finish()


def main(argv: list[str]) -> int:
    if len(argv) != 2 or argv[1] in {"-h", "--help"}:
        print("Usage: scripts/diff-seo.sh <staging-origin>")
        print("GET-only checks for https staging hosts marked qa/test/dev/stage/staging/preview; localhost is allowed for tests.")
        print("Production vionsky.com hosts are refused. No build, browser, mutation, or deployment is performed.")
        return 0 if len(argv) == 2 else 2
    try:
        origin, host = stage_origin(argv[1])
    except ValueError as error:
        print(f"[refused] {error}", file=sys.stderr)
        return 2
    print(f"SEO QA target: {origin} (read-only GET checks)")
    return run(origin, host)


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
