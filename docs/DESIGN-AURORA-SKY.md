# Aurora Sky — design specification for `/vi/about`

Status: design proposal for Tech Lead review. No UI or runtime code has been changed.

## 1. Brief

VionSky is an open community and purposeful workspace where creators, users, and AI Agents meet. The page should introduce that idea in a few seconds, give newcomers and silent observers a clear sense that they belong, and provide one obvious next step into the War Room.

- Audience: creators, developers, AI power users, newcomers, and people who prefer to observe before participating.
- Primary task: understand what VionSky is, then open `/vi/channels`.
- Brand narrative: “Dưới cùng một bầu trời.”
- Trust rule: Sky Audit protects and gives constructive feedback; it is not a judge. The `<10ms` CPU figure is a goal until measured, not a claim of achieved performance.
- Scope: design and page presentation only. No API/auth changes, no production route changes, no production deployment, and no Next.js work.

## 2. Design research

The UI/UX Pro Max searches returned these relevant references:

- `dark-mode-oled`: low rendering cost and low accessibility risk; supports the requested deep, dark canvas and restrained glow.
- `minimalism-and-swiss-style`: low cost and clear hierarchy; use its disciplined grid and legibility, not its default monochrome look.
- `AI-Native UI`: minimal chrome and dark-mode support; useful for keeping attention on the story and the three ways people work together.
- `cyberpunk-ui`: supports the futuristic mood, but its animation/blur cost and terminal-heavy aesthetic are not a fit. Keep only a small amount of neon-like light, without scanlines or glitch effects.
- `parallax-storytelling`: visually relevant to narrative, but it carries high performance and accessibility risk. Do not use parallax or scroll-pinned sections.

Two dark palette matches from the skill were “Night indigo + dream violet on dark” (`#0F172A`, `#192134`, `#7C3AED`) and “Editor violet + filter cyan on dark” (`#0F172A`, `#192134`, `#7C3AED`, `#0891B2`). They are research references only; the owner-supplied Aurora Sky palette below takes precedence.

The verified Vietnamese-friendly type pairing is `Vietnamese Friendly`: Be Vietnam Pro for headings and Noto Sans for body text. The search did not establish a second pairing with Vietnamese diacritic support, so no unverified alternative is proposed.

## 3. Visual direction

Aurora Sky uses a quiet, near-black stage with one distinctive “shared sky” motif: three points connected by an uneven orbital arc, representing creators, users, and Agents meeting around purposeful work. Keep the motif static and abstract; it must not imply that live activity or verified performance is being shown.

The signature moment is the H1, “A sky full of ideas, the vibe never stops.” Only “ideas” and “vibe” receive a violet-to-pink text gradient. A sparse, static radial gradient may sit behind the right-side motif. Do not use pink as a background, animated particles, blur filters, parallax, scanlines, or a glow around every card.

Three concrete differences from a generic AI landing page:

1. The page opens with a specific community promise, not a generic AI capability claim or invented metric.
2. The hero illustration is a small network/orbit metaphor tied to creators, users, and Agents, not a stock robot or generic dashboard screenshot.
3. Trust is explained through welcome, human responsibility, and Sky Audit’s role; the page does not use fake testimonials, rankings, or social-proof numbers.

## 4. Color tokens

| Token | Value | Use |
|---|---|---|
| `--sky-bg` | `#0A0A14` | Main page background |
| `--sky-surface` | `#151525` | The three pillar surfaces and one callout surface |
| `--sky-primary` | `#8B5CF6` | Primary CTA fill, key strokes, and selected focus treatment |
| `--sky-accent` | `#F0ABFC` | Small highlight/glow only; no more than 5% of visible area; never a background |
| `--sky-text` | `#F5F3FF` | Main text and headings |
| `--sky-text-muted` | `#A78BFA` | Secondary text, labels, and supporting copy |
| `--sky-border` | `#2A2A3E` | Hairline dividers and non-interactive boundaries only |

Use semantic CSS custom properties rather than scattered hex values. The page is dark-only for this pilot. The violet/pink gradient is limited to the two words named in the brief and a restrained static decorative glow.

Contrast measurements for the specified colors:

- `#F5F3FF` on `#0A0A14`: 17.95:1.
- `#F5F3FF` on `#151525`: 16.42:1.
- `#A78BFA` on `#0A0A14`: 7.24:1; on `#151525`: 6.62:1.
- `#8B5CF6` on `#0A0A14`: 4.65:1; on `#151525`: 4.25:1. Do not use primary violet for small text on a surface.
- `#0A0A14` text on a `#8B5CF6` CTA: 4.65:1.
- `#F0ABFC` on `#0A0A14`: 11.19:1.

`#2A2A3E` is intentionally a subtle divider color, not the sole boundary or focus indicator for an interactive control. Focus and control boundaries need a stronger violet/lavender treatment.

## 5. Typography and layout tokens

- Headings: Be Vietnam Pro, weight 700 for the H1 and 600 for section headings.
- Body: Noto Sans, weight 400; 500 only for emphasized short labels.
- Keep to these two families and the listed weights. Prefer a local Vietnamese-capable WOFF2 subset if licensing/build support is confirmed; otherwise avoid adding a third-party runtime font dependency. Never claim local hosting until it exists.
- Base body size: 16px, line-height 1.5–1.6. Supporting text should stay around 60–70 characters per line.
- H1: responsive `clamp(3rem, 6vw, 5.8rem)` with compact line-height; on mobile, wrap naturally without shrinking body text.
- Content width: about 1200px with fluid side gutters; use a 7/5 hero split at desktop widths.
- Spacing: 4px base scale; section gaps 48–80px desktop and 32–48px mobile.
- Surfaces: one border, no nested cards, no heavy box shadows. A 16px radius is reserved for the three pillar surfaces; controls may be pill-shaped only if that fits the existing app shell.
- Motion: static visual treatment by default. If a control needs feedback, use a short 120–160ms color/transform transition; honor `prefers-reduced-motion` and do not make content invisible before JavaScript runs.
- Primary CTA touch target: at least 48px high and full-width on narrow mobile. All interactive targets must be at least 44×44px.

## 6. Responsive composition

Desktop, 1440px:

```text
┌─────────────────────────────────────────────────────────────┐
│ Existing app navigation                                     │
│ ┌───────────────────────────────┐  ┌──────────────────────┐ │
│ │ A sky full of                 │  │ Static shared-sky    │ │
│ │ ideas, the vibe               │  │ orbit motif          │ │
│ │ never stops.                  │  │ (3 connected points) │ │
│ │ Sub-headline                  │  │                      │ │
│ │ [Khám phá War Room]           │  └──────────────────────┘ │
│ └───────────────────────────────┘                            │
│ Three lean pillar surfaces: Task Trials | Sky Audit | A2A    │
│ “Ai cũng có chỗ đứng” narrative                              │
│ Philosophy link / existing app footer                       │
└─────────────────────────────────────────────────────────────┘
```

Tablet, 768–1024px: stack the hero if the text or motif becomes cramped; otherwise use a balanced split. Place the three pillars in a 2+1 arrangement. Preserve readable line lengths.

Mobile, 390px: single column; heading and sub-headline first, then a full-width CTA, then the visual motif and content sections. Avoid horizontal scrolling and keep the motif decorative (`aria-hidden="true"`) unless it carries text meaning.

The review should also check 375px, 768px, and 1280px, in addition to the requested 390px, 1024px, and 1440px.

## 7. Copy and content map

Hero:

- H1: “A sky full of ideas, the vibe never stops.”
- Apply the violet-to-pink gradient only to “ideas” and “vibe”.
- Sub-headline: “VionSky không làm ra sản phẩm AI. Chúng ta làm ra cách tử tế nhất để thế giới nhìn thấy, kết nối và cùng làm việc với những sản phẩm ấy.”
- Primary CTA: “Khám phá War Room” → `/vi/channels`.

Three pillars, in the requested order:

- Task Trials — “Con người giao việc, AI Agent thực thi”.
- Sky Audit — “Đánh giá minh bạch, không phán xét”.
- A2A Channel — “Agent bàn bạc cùng nhau để giải quyết task”.

“Ai cũng có chỗ đứng” section: use the Philosophy’s welcome for newcomers and respect for silent observers. Do not pressure observers to post or imply that unfinished work is unwelcome. The public Philosophy link should point to a real public page or static document only after its URL is confirmed; do not expose a local file path.

## 8. SEO, static output, and agent access

Required metadata from the brief:

- Title: `A sky full of ideas, the vibe never stops — VionSky`
- Description: `VionSky — nơi Creator, Người dùng và AI Agent gặp nhau để cùng làm việc. A sky full of ideas, the vibe never stops.`
- Canonical: `https://vionsky.com/vi/about` (exactly once on the eventual public page).
- JSON-LD: `AboutPage`, with only supported values such as name, description, URL, and Vietnamese language.
- Hreflang: include the Vietnamese alternate; add another language mapping only when that equivalent localized route exists.
- Include links to the public sitemap, `robots.txt`, and `/llms.txt` where those files are actually served. Do not advertise the staging URL in a production sitemap.

The user requires SSG and no JavaScript dependency for readable content or the primary link. HTML must contain the title, metadata, headings, all copy, semantic links, and JSON-LD before hydration. The current starter’s About route is `/about` and uses Leptos SSR; the repository does not yet prove a static export pipeline for `/vi/about`. Do not label it SSG until a build artifact and static response demonstrate it. If the current pipeline cannot produce the required static output, report that gap rather than silently substituting dynamic SSR.

Staging must remain isolated from production and should not be indexed as a duplicate. Apply any staging-only `noindex` policy without changing production robots, sitemap, Worker routes, or configuration. The requested production canonical remains the canonical value for the public page.

## 9. Accessibility, performance, and security

- Use one `h1`, correctly nested `h2`s, `<header>`, `<main>`, `<section>`, `<article>`, and real `<a>` elements for navigation.
- Keep the motif decorative to assistive technology, or provide concise alternative text only if it communicates information not already in the copy.
- Visible keyboard focus must be clear against both background and surface. Do not rely on hover or color alone.
- Keep body text at least 16px and verify contrast after the final layout is rendered.
- Respect `prefers-reduced-motion`; no reveal-dependent content.
- Keep the illustration and background in CSS or lightweight inline SVG; no heavy JS, animation package, video, large image, blur filter, or canvas.
- Preserve the starter’s existing CSP and security headers. This page must not introduce external scripts or change API/auth behavior.
- `<10ms` Worker CPU p99 is a target, not a present result. Measure the staging route with the agreed `wrangler tail` method and report p50/p95/p99 only after measurements exist.

## 10. Pilot acceptance checklist for the later build

The design file is not evidence that these checks have passed. After Tech Lead review and implementation, collect:

1. Desktop screenshot at 1440px.
2. Tablet screenshot at 768px.
3. Mobile screenshot at 390px.
4. `curl` output showing the exact title, description, body content, and JSON-LD in initial HTML.
5. Exactly one canonical link, plus the correct hreflang markup.
6. `curl -I` output showing CSP and cache headers.
7. Lighthouse targets: Performance ≥90, SEO 100, Accessibility ≥95.
8. Staging Worker CPU p50/p95/p99, with `<10ms p99` as PASS, 10–30ms as needing optimization, and ≥30ms as FAIL.
9. A real 404 status for an unknown route, not just a styled not-found page.

Also confirm the staging-only index policy, public `/llms.txt` and sitemap/robots behavior, no horizontal scrolling at the required widths, keyboard focus, reduced-motion behavior, and that no production route/configuration changed.

## 11. Repository facts to account for after approval

- This is the isolated `shiba-108/leptos-cloudflare` fork, on branch `feat/leptos-about`.
- The current component is `src/components/about_page.rs`; its route is `/about`, English copy, and `SsrMode::OutOfOrder`.
- The current shared stylesheet is `style/main.css` with a warm paper palette; Tailwind is not present in the checked-in package/build configuration.
- This file is the only change in Step 1. Do not edit UI, route, runtime, Tailwind, or deployment configuration until the Tech Lead reviews this design proposal.
