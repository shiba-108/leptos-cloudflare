#!/usr/bin/env python3
"""Offline integration tests for diff-seo.sh; uses only a local HTTP fixture."""
from __future__ import annotations

import subprocess
import threading
import unittest
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "scripts" / "diff-seo.sh"
TITLE = "A sky full of ideas, the vibe never stops — VionSky"
DESCRIPTION = "VionSky — nơi Creator, Người dùng và AI Agent gặp nhau để cùng làm việc. A sky full of ideas, the vibe never stops."
CANONICAL = "https://vionsky.com/vi/about"


class Fixture:
    def __init__(self) -> None:
        self.mode = "valid"
        self.requests: list[tuple[str, str]] = []


fixture = Fixture()


class Handler(BaseHTTPRequestHandler):
    protocol_version = "HTTP/1.1"

    def log_message(self, *_args) -> None:
        pass

    def send(self, status: int, body: str, content_type: str) -> None:
        encoded = body.encode("utf-8")
        self.send_response(status)
        self.send_header("Content-Type", content_type)
        self.send_header("Content-Length", str(len(encoded)))
        self.end_headers()
        self.wfile.write(encoded)

    def do_GET(self) -> None:
        fixture.requests.append(("GET", self.path))
        mode = fixture.mode
        if self.path == "/vi/about":
            if mode == "redirect":
                self.send_response(302)
                self.send_header("Location", f"http://127.0.0.1:{self.server.server_port}/redirect-target")
                self.send_header("Content-Length", "0")
                self.end_headers()
                return
            noindex = "<meta name=\"robots\" content=\"noindex, nofollow\">"
            description = f'<meta name="description" content="{DESCRIPTION}">'
            canonical = f'<link rel="canonical" href="{CANONICAL}">'
            alternate = f'<link rel="alternate" hreflang="vi" href="{CANONICAL}">'
            if mode == "bad-page":
                noindex = ""
                description = '<meta name="description" content="wrong">'
                canonical += '<link rel="canonical" href="https://wrong.example/">'
                alternate = ""
            jsonld = (
                '{"@context":"https://schema.org","@type":"AboutPage",'
                f'"name":"{TITLE}","description":"{DESCRIPTION}",'
                f'"url":"{CANONICAL}","inLanguage":"vi"'
                '}'
            )
            if mode == "bad-page":
                jsonld = "{not-json"
            html = f'''<!doctype html><html lang="vi"><head><meta charset="utf-8">
<title>{TITLE}</title>{description}{canonical}{alternate}{noindex}
<script type="application/ld+json">{jsonld}</script></head><body><main>
<h1>A sky full of <span>ideas</span>, the vibe never stops.</h1>
<p>VionSky không làm ra sản phẩm AI. Chúng ta làm ra cách tử tế nhất...</p>
<a href="/vi/channels">Khám phá War Room</a>
<section>Task Trials — Con người giao việc, AI Agent thực thi</section>
<section>Sky Audit — Đánh giá minh bạch, không phán xét</section>
<section>A2A Channel — Agent bàn bạc cùng nhau để giải quyết task</section>
<section>Ai cũng có chỗ đứng</section></main></body></html>'''
            self.send(200, html, "text/html; charset=utf-8")
        elif self.path == "/sitemap.xml":
            if mode == "bad-resources":
                self.send(200, "<broken", "application/xml")
            elif mode == "staging-sitemap":
                stage_url = f"http://127.0.0.1:{self.server.server_port}/vi/about"
                self.send(200, f'<urlset><url><loc>{stage_url}</loc></url></urlset>', "application/xml")
            else:
                self.send(200, '<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9"><url><loc>https://vionsky.com/vi/about</loc></url></urlset>', "application/xml; charset=utf-8")
        elif self.path == "/robots.txt":
            if mode == "bad-resources":
                self.send(404, "not found", "text/plain")
            else:
                self.send(200, "User-agent: *\nDisallow: /admin\nSitemap: https://vionsky.com/sitemap.xml\n", "text/plain; charset=utf-8")
        elif self.path == "/llms.txt":
            if mode == "bad-resources":
                self.send(404, "not found", "text/plain")
            else:
                self.send(200, "# VionSky\nCreators, users, and AI agents work together.\n", "text/plain; charset=utf-8")
        elif mode == "false-404":
            self.send(200, "<html><body>Not found</body></html>", "text/html")
        else:
            self.send(404, "Not found", "text/plain")

    def do_POST(self) -> None:
        fixture.requests.append(("POST", self.path))
        self.send(405, "Method not allowed", "text/plain")


class SeoQaTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.server = ThreadingHTTPServer(("127.0.0.1", 0), Handler)
        cls.thread = threading.Thread(target=cls.server.serve_forever, daemon=True)
        cls.thread.start()
        cls.origin = f"http://127.0.0.1:{cls.server.server_port}"

    @classmethod
    def tearDownClass(cls) -> None:
        cls.server.shutdown()
        cls.server.server_close()
        cls.thread.join(timeout=2)

    def run_check(self, mode: str = "valid", target: str | None = None) -> subprocess.CompletedProcess[str]:
        fixture.mode = mode
        fixture.requests.clear()
        return subprocess.run(
            ["bash", str(SCRIPT), target or self.origin],
            cwd=ROOT,
            text=True,
            capture_output=True,
            timeout=30,
            check=False,
        )

    def test_valid_fixture_passes_all_requested_read_only_checks(self) -> None:
        result = self.run_check()
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        for label in ("title", "description", "canonical", "hreflang", "JSON-LD", "sitemap.xml", "robots.txt", "llms.txt", "raw HTML", "real 404", "staging noindex"):
            self.assertIn(label, result.stdout)
        self.assertIn("does not prove SSG", result.stdout)
        self.assertTrue(fixture.requests)
        self.assertTrue(all(method == "GET" for method, _path in fixture.requests))

    def test_metadata_and_noindex_regressions_fail(self) -> None:
        result = self.run_check("bad-page")
        self.assertEqual(result.returncode, 1, result.stdout + result.stderr)
        for label in ("description", "canonical", "hreflang", "JSON-LD", "staging noindex"):
            self.assertIn(f"[FAIL] {label}", result.stdout)

    def test_missing_resources_and_styled_200_not_found_fail(self) -> None:
        result = self.run_check("bad-resources")
        self.assertEqual(result.returncode, 1, result.stdout + result.stderr)
        self.assertIn("[FAIL] sitemap XML", result.stdout)
        self.assertIn("[FAIL] robots.txt HTTP", result.stdout)
        self.assertIn("[FAIL] llms.txt HTTP", result.stdout)

        result = self.run_check("false-404")
        self.assertEqual(result.returncode, 1, result.stdout + result.stderr)
        self.assertIn("[FAIL] real 404", result.stdout)

    def test_staging_urls_in_sitemap_fail(self) -> None:
        result = self.run_check("staging-sitemap")
        self.assertEqual(result.returncode, 1, result.stdout + result.stderr)
        self.assertIn("[FAIL] sitemap staging isolation", result.stdout)

    def test_redirect_is_reported_without_following_it(self) -> None:
        result = self.run_check("redirect")
        self.assertEqual(result.returncode, 1, result.stdout + result.stderr)
        self.assertIn("[FAIL] about page HTTP: GET /vi/about returned 302", result.stdout)
        self.assertNotIn(("GET", "/redirect-target"), fixture.requests)

    def test_production_host_is_refused_before_any_request(self) -> None:
        result = self.run_check(target="https://vionsky.com")
        self.assertEqual(result.returncode, 2)
        self.assertIn("refusing the production VionSky host", result.stderr)
        self.assertEqual(fixture.requests, [])

    def test_remote_host_without_staging_marker_is_refused(self) -> None:
        result = self.run_check(target="https://vionsky.example")
        self.assertEqual(result.returncode, 2)
        self.assertIn("not recognizably staging", result.stderr)
        self.assertEqual(fixture.requests, [])


if __name__ == "__main__":
    unittest.main(verbosity=2)
