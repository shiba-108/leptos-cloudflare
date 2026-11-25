# API Contract — VionSky (bản chuẩn cho Frontend / Dioxus)

> **Cập nhật:** 2026-11-25 · **Nguồn:** code `main@a50d0e0` + Cloudflare Rulesets API.
> Đây là hợp đồng duy nhất cho Culi; thay thế `API_CONTRACT.md` ở root (bản cũ có vài chỗ lệch code).
> Mọi dòng đều có `file:line` hoặc URL để đối chiếu. Không suy diễn từ tên route.

## 0. Quick start

```ts
// UI/Dioxus web: LUÔN dùng cookie, cùng origin. Không đọc token, không gắn Authorization.
const res = await fetch('/api/games?category=all&limit=20', {
  credentials: 'same-origin',
  cache: 'no-store',
});
const json = await res.json();

// Agent/script (không phải browser): Bearer JWT hoặc API key
fetch('https://vionsky.com/api/v1/products', {
  method: 'POST',
  headers: { 'Content-Type': 'application/json', 'Authorization': `Bearer ${API_KEY}` },
  body: JSON.stringify({ play_url: 'https://...' }),
});
```

- Edge rate-limit: **30 request / 10s / IP+côlo** cho mọi path chứa `/api/`,
  **ngoại trừ** `/api/auth/session` và `/api/notifications*` (đã sửa 2026-11-24).
  Nếu gặp 429: đọc `retry_after_seconds` (hoặc `Retry-After`), backoff, không retry ngay.
- Lỗi luôn nằm ở body JSON; đọc `error` trước, sau đó `message` nếu có.
- Không có WebSocket cho browser. Realtime = **HTTP long-poll** hoặc **SSE**. Xem mục 5.

## 1. Endpoints (94 method+path duy nhất)

Nguồn gom: **82 method Next** (`src/app/api/**/route.ts`) + **41 entry Rust**
(`rust-workers/src/lib.rs`, trong đó có route forward sang Next).

Ký hiệu Auth: `public` · `user` (cookie/Bearer) · `mod` · `admin` · `owner` ·
`room-secret` (`x-room-secret`) · `cron-secret` · `api-key`.
Layer: `Rust` = handler trong Worker Rust; `Next` = route Next; `Rust → Next` = Rust forward qua Service Binding.

| Method | Path | Auth | Rate limit | Layer | Source |
|---|---|---|---|---|---|
| ANY | `/*catchall` | varies | none at Rust layer | Rust | `rust-workers/src/lib.rs:6868` |
| GET | `/api/admin/analytics` | moderator (or admin) | — | Rust | `rust-workers/src/lib.rs:6798 (handler rust-workers/src/lib.rs:4137)` |
| GET | `/api/admin/audit-logs` | admin (ADMIN_EMAIL) | — | Rust | `rust-workers/src/lib.rs:6791 (handler rust-workers/src/lib.rs:4058)` |
| DELETE | `/api/admin/games` | moderator (GET) / admin (PATCH,DELETE) | — | Rust | `rust-workers/src/lib.rs:6781 GET / rust-workers/src/lib.rs:6810 PATCH / rust-workers/src/lib.rs:6812 DELETE (handlers rust-workers/src/lib.rs:3765, rust-workers/src/lib.rs:5131, rust-workers/src/lib.rs:5378)` |
| GET | `/api/admin/games` | moderator (GET) / admin (PATCH,DELETE) | — | Rust | `rust-workers/src/lib.rs:6781 GET / rust-workers/src/lib.rs:6810 PATCH / rust-workers/src/lib.rs:6812 DELETE (handlers rust-workers/src/lib.rs:3765, rust-workers/src/lib.rs:5131, rust-workers/src/lib.rs:5378)` |
| PATCH | `/api/admin/games` | moderator (GET) / admin (PATCH,DELETE) | — | Rust | `rust-workers/src/lib.rs:6781 GET / rust-workers/src/lib.rs:6810 PATCH / rust-workers/src/lib.rs:6812 DELETE (handlers rust-workers/src/lib.rs:3765, rust-workers/src/lib.rs:5131, rust-workers/src/lib.rs:5378)` |
| GET | `/api/admin/report` | moderator (or admin) | — | Rust | `rust-workers/src/lib.rs:6800 (handler rust-workers/src/lib.rs:4452)` |
| POST | `/api/admin/report/proposals/[id]` | admin (ADMIN_EMAIL) | — | Rust | `rust-workers/src/lib.rs:6807 (handler rust-workers/src/lib.rs:4947)` |
| DELETE | `/api/admin/reports` | moderator | — | Rust | `rust-workers/src/lib.rs:6785 GET / rust-workers/src/lib.rs:6803 DELETE (handlers rust-workers/src/lib.rs:3877, rust-workers/src/lib.rs:4787)` |
| GET | `/api/admin/reports` | moderator | — | Rust | `rust-workers/src/lib.rs:6785 GET / rust-workers/src/lib.rs:6803 DELETE (handlers rust-workers/src/lib.rs:3877, rust-workers/src/lib.rs:4787)` |
| DELETE | `/api/admin/reviews` | moderator | — | Rust | `rust-workers/src/lib.rs:6787 GET / rust-workers/src/lib.rs:6805 DELETE (handlers rust-workers/src/lib.rs:3914, rust-workers/src/lib.rs:4854)` |
| GET | `/api/admin/reviews` | moderator | — | Rust | `rust-workers/src/lib.rs:6787 GET / rust-workers/src/lib.rs:6805 DELETE (handlers rust-workers/src/lib.rs:3914, rust-workers/src/lib.rs:4854)` |
| GET | `/api/admin/stats` | moderator (or admin) | — | Rust | `rust-workers/src/lib.rs:6783 (handler rust-workers/src/lib.rs:3812)` |
| GET | `/api/admin/system-health` | admin (ADMIN_EMAIL) | none (KV cache key syshealth:24h:v1 TTL 300s, fail-open on KV error) | Rust | `rust-workers/src/lib.rs:6796 (handler rust-workers/src/lib.rs:6128)` |
| GET | `/api/admin/users` | admin (ADMIN_EMAIL) | — | Rust | `rust-workers/src/lib.rs:6789 (handler rust-workers/src/lib.rs:3955)` |
| PATCH | `/api/admin/users/[id]/role` | admin (ADMIN_EMAIL) | — | Rust | `rust-workers/src/lib.rs:6814 (handler rust-workers/src/lib.rs:5456)` |
| POST | `/api/auth/oauth` | public | — | Next | `src/app/api/auth/oauth/route.ts:35-54` |
| GET | `/api/auth/session` | public | — | Next | `src/app/api/auth/session/route.ts:5-13` |
| POST | `/api/auth/sign-in` | public | — | Next | `src/app/api/auth/sign-in/route.ts:4-20` |
| POST | `/api/auth/sign-out` | user | — | Next | `src/app/api/auth/sign-out/route.ts:4-11` |
| POST | `/api/auth/sign-up` | public | — | Next | `src/app/api/auth/sign-up/route.ts:4-23` |
| GET | `/api/bridge/history` | room-secret | Next.js | Rust | `rust-workers/src/lib.rs:6440` |
| GET | `/api/bridge/info` | room-secret (query room / header) | Next.js | Rust | `rust-workers/src/lib.rs:6438` |
| GET | `/api/bridge/poll` | room-secret | Next.js | Rust | `rust-workers/src/lib.rs:6442` |
| POST | `/api/bridge/send` | room-secret | Next.js | Rust | `rust-workers/src/lib.rs:6438` |
| GET | `/api/cover-proxy` | public | — | Rust | `rust-workers/src/lib.rs:6675 (handler rust-workers/src/lib.rs:7643)` |
| GET | `/api/creator/feedback` | user (owner/creator) | — | Rust | `rust-workers/src/lib.rs:6719 (handler rust-workers/src/lib.rs:964)` |
| POST | `/api/creator/feedback/handle` | user (owner only) | — | Rust | `rust-workers/src/lib.rs:6721 (handler rust-workers/src/lib.rs:1130)` |
| GET | `/api/creator/feedback/settings` | user (owner) | — | Rust | `rust-workers/src/lib.rs:6723 GET / rust-workers/src/lib.rs:6725 PATCH (handlers rust-workers/src/lib.rs:1179, rust-workers/src/lib.rs:1194)` |
| PATCH | `/api/creator/feedback/settings` | user (owner) | — | Rust | `rust-workers/src/lib.rs:6723 GET / rust-workers/src/lib.rs:6725 PATCH (handlers rust-workers/src/lib.rs:1179, rust-workers/src/lib.rs:1194)` |
| GET | `/api/creator/games` | user | — | Next | `src/app/api/creator/games/route.ts:63-113` |
| PATCH | `/api/creator/games` | owner | — | Next | `src/app/api/creator/games/route.ts:122-255` |
| POST | `/api/creator/games/[id]/re-audit` | owner | 3 / 1 day UTC ((custom: D1 daily_audit_count), per gameId) | Next | `src/app/api/creator/games/[id]/re-audit/route.ts:47-179` |
| POST | `/api/creator/send-digest` | cron-secret | — | Next | `src/app/api/creator/send-digest/route.ts:25-74` |
| GET | `/api/games` | public (GET status!=approved requires admin) | — | Rust | `rust-workers/src/lib.rs:6693 GET / rust-workers/src/lib.rs:6696 POST / rust-workers/src/lib.rs:6698 PATCH (handlers rust-workers/src/lib.rs:7917)` |
| PATCH | `/api/games` | admin | — | Rust → Next | `src/app/api/games/route.ts:228-314` |
| POST | `/api/games` | user | 2 / 1 day (UTC) (game_submit, per userId) | Rust → Next | `src/app/api/games/route.ts:183-221` |
| DELETE | `/api/games/[id]` | mod | — | Next | `src/app/api/games/[id]/route.ts:33-159` |
| POST | `/api/games/[id]/play` | public | — | Next | `src/app/api/games/[id]/play/route.ts:43-97` |
| GET | `/api/games/lookup` | public | 60 requests / 60s per IP, scope 'search', KV RATE_LIMIT_KV, fail-closed 429 | Rust | `rust-workers/src/lib.rs:6691 (handler rust-workers/src/lib.rs:520)` |
| DELETE | `/api/notifications` | user | — | Rust | `rust-workers/src/lib.rs:6754 GET / rust-workers/src/lib.rs:6756 DELETE (handlers rust-workers/src/lib.rs:2878, rust-workers/src/lib.rs:2949)` |
| GET | `/api/notifications` | user | — | Rust | `rust-workers/src/lib.rs:6754 GET / rust-workers/src/lib.rs:6756 DELETE (handlers rust-workers/src/lib.rs:2878, rust-workers/src/lib.rs:2949)` |
| PATCH | `/api/notifications/read` | user | — | Rust | `rust-workers/src/lib.rs:6758 (handler rust-workers/src/lib.rs:3009)` |
| GET | `/api/og-fallback` | public | — | Next | `src/app/api/og-fallback/route.ts:12-29` |
| GET | `/api/ping` | public | — | Rust | `rust-workers/src/lib.rs:6679` |
| GET | `/api/posts` | user (GET optional viewer; feed=following needs login) | 3 posts / 600s / user, scope 'post_create', KV, fail-closed 429; admin bypass flag applies | Rust | `rust-workers/src/lib.rs:6714 POST / rust-workers/src/lib.rs:6735 GET (handlers rust-workers/src/lib.rs:2131, rust-workers/src/lib.rs:7451)` |
| POST | `/api/posts` | user (GET optional viewer; feed=following needs login) | 3 posts / 600s / user, scope 'post_create', KV, fail-closed 429; admin bypass flag applies | Rust | `rust-workers/src/lib.rs:6714 POST / rust-workers/src/lib.rs:6735 GET (handlers rust-workers/src/lib.rs:2131, rust-workers/src/lib.rs:7451)` |
| DELETE | `/api/posts/[id]` | owner | — | Rust → Next | `src/app/api/posts/[id]/route.ts:273-340` |
| GET | `/api/posts/[id]` | public (optional Bearer viewer) | — | Rust | `rust-workers/src/lib.rs:6742 GET / rust-workers/src/lib.rs:6744 PATCH / rust-workers/src/lib.rs:6746 DELETE (handler rust-workers/src/lib.rs:8112)` |
| PATCH | `/api/posts/[id]` | owner | — | Rust → Next | `src/app/api/posts/[id]/route.ts:106-264` |
| GET | `/api/posts/[id]/comments` | user (POST); public (GET) | 10 comments / 60s / user, scope 'post_comment', KV, fail-closed 429 | Rust | `rust-workers/src/lib.rs:6727 POST / rust-workers/src/lib.rs:6739 GET (handlers rust-workers/src/lib.rs:2504, rust-workers/src/lib.rs:7759)` |
| POST | `/api/posts/[id]/comments` | user (POST); public (GET) | 10 comments / 60s / user, scope 'post_comment', KV, fail-closed 429 | Rust | `rust-workers/src/lib.rs:6727 POST / rust-workers/src/lib.rs:6739 GET (handlers rust-workers/src/lib.rs:2504, rust-workers/src/lib.rs:7759)` |
| DELETE | `/api/posts/[id]/comments/[commentId]` | owner | — | Next | `src/app/api/posts/[id]/comments/[commentId]/route.ts:125-204` |
| PATCH | `/api/posts/[id]/comments/[commentId]` | owner | — | Next | `src/app/api/posts/[id]/comments/[commentId]/route.ts:25-117` |
| POST | `/api/posts/[id]/upvote` | user | 100 post-upvotes / day / user, scope 'post_upvote', KV, fail-closed 429 | Rust | `rust-workers/src/lib.rs:6709 (handler rust-workers/src/lib.rs:1887)` |
| POST | `/api/reports` | user | 5 / 1 day (report, per userId) | Next | `src/app/api/reports/route.ts:30-101` |
| GET | `/api/reviews` | user (POST); public (GET) | 3 reviews / 600s per (user:game), scope 'review', KV, fail-closed 429 | Rust | `rust-workers/src/lib.rs:6716 POST / rust-workers/src/lib.rs:6737 GET (handlers rust-workers/src/lib.rs:2318, rust-workers/src/lib.rs:7852)` |
| POST | `/api/reviews` | user (POST); public (GET) | 3 reviews / 600s per (user:game), scope 'review', KV, fail-closed 429 | Rust | `rust-workers/src/lib.rs:6716 POST / rust-workers/src/lib.rs:6737 GET (handlers rust-workers/src/lib.rs:2318, rust-workers/src/lib.rs:7852)` |
| GET | `/api/rooms` | user (GET/POST); owner for room ops; room-secret for agent ops | Next.js | Rust | `rust-workers/src/lib.rs:6427` |
| POST | `/api/rooms` | user (GET/POST); owner for room ops; room-secret for agent ops | Next.js | Rust | `rust-workers/src/lib.rs:6427` |
| DELETE | `/api/rooms/[roomId]` | owner | — | Next | `src/app/api/rooms/[roomId]/route.ts:44-68` |
| PATCH | `/api/rooms/[roomId]` | owner | — | Next | `src/app/api/rooms/[roomId]/route.ts:9-35` |
| GET | `/api/rooms/[roomId]/agents` | room-secret | — | Next | `src/app/api/rooms/[roomId]/agents/route.ts:8-24` |
| POST | `/api/rooms/[roomId]/agents` | room-secret | — | Next | `src/app/api/rooms/[roomId]/agents/route.ts:34-72` |
| DELETE | `/api/rooms/[roomId]/agents/[agentId]` | room-secret | — | Next | `src/app/api/rooms/[roomId]/agents/[agentId]/route.ts:8-28` |
| PATCH | `/api/rooms/[roomId]/agents/[agentId]/rename` | room-secret | — | Next | `src/app/api/rooms/[roomId]/agents/[agentId]/rename/route.ts:12-48` |
| GET | `/api/rooms/[roomId]/messages` | owner | — | Next | `src/app/api/rooms/[roomId]/messages/route.ts:26-71` |
| GET | `/api/rooms/[roomId]/presence` | room-secret | — | Next | `src/app/api/rooms/[roomId]/presence/route.ts:8-38` |
| POST | `/api/rooms/[roomId]/rotate` | owner | — | Next | `src/app/api/rooms/[roomId]/rotate/route.ts:19-40` |
| DELETE | `/api/rooms/[roomId]/sticky` | room-secret | — | Next | `src/app/api/rooms/[roomId]/sticky/route.ts:65-89` |
| GET | `/api/rooms/[roomId]/sticky` | room-secret | — | Next | `src/app/api/rooms/[roomId]/sticky/route.ts:13-34` |
| POST | `/api/rooms/[roomId]/sticky` | room-secret | — | Next | `src/app/api/rooms/[roomId]/sticky/route.ts:36-63` |
| GET | `/api/search` | public | 30 requests / 60s per IP, scope 'search', KV, fail-closed 429 | Rust | `rust-workers/src/lib.rs:6701 (handler rust-workers/src/lib.rs:762)` |
| POST | `/api/submit/analyze` | user | Next.js ANALYZE_LIMIT_PER_HOUR | Rust | `rust-workers/src/lib.rs:6424` |
| POST | `/api/upload` | user | 5 / 1 day (community_image_upload, per userId) | Next | `src/app/api/upload/route.ts:36-132` |
| GET | `/api/upvotes` | user | — | Rust → Next | `src/app/api/upvotes/route.ts:45-67` |
| POST | `/api/upvotes` | user | 100 upvotes / day / user, scope 'upvote', KV, fail-closed 429; admin bypass if RATE_LIMIT_ROLE_BYPASS=true | Rust | `rust-workers/src/lib.rs:6707 POST / rust-workers/src/lib.rs:6775 GET forward (handler rust-workers/src/lib.rs:1753)` |
| GET | `/api/upvotes/batch` | public | — | Rust | `rust-workers/src/lib.rs:6685 (handler rust-workers/src/lib.rs:447)` |
| GET | `/api/users/[id]` | public (optional Bearer viewer) | — | Rust | `rust-workers/src/lib.rs:6769 (handler rust-workers/src/lib.rs:3174)` |
| POST | `/api/users/[id]/follow` | user | 20 follows / day / user, scope 'follow', KV, fail-closed 429 — admin NOT bypassed | Rust | `rust-workers/src/lib.rs:6760 (handler rust-workers/src/lib.rs:3406)` |
| DELETE | `/api/users/me` | user | — | Rust → Next | `src/app/api/users/me/route.ts:193-258` |
| GET | `/api/users/me` | user | — | Rust | `rust-workers/src/lib.rs:6762 GET / rust-workers/src/lib.rs:6765 PATCH / rust-workers/src/lib.rs:6767 DELETE (handler rust-workers/src/lib.rs:7979)` |
| PATCH | `/api/users/me` | user | — | Rust → Next | `src/app/api/users/me/route.ts:86-177` |
| POST | `/api/users/sync` | user | Next.js rate-limit (ANALYZE-like) | Rust | `rust-workers/src/lib.rs:6424` |
| POST | `/api/v1/comments` | user | 10 / 1 minute (post_comment, per userId) | Next | `src/app/api/v1/comments/route.ts:193-196` |
| DELETE | `/api/v1/keys` | user | — | Next | `src/app/api/v1/keys/route.ts:177-215` |
| GET | `/api/v1/keys` | user | — | Next | `src/app/api/v1/keys/route.ts:40-57` |
| POST | `/api/v1/keys` | user | 20 / 10 minutes (apikey_create, per apikey:***) | Next | `src/app/api/v1/keys/route.ts:71-170` |
| GET | `/api/v1/openapi.json` | public | — | Next | `src/app/api/v1/openapi.json/route.ts:311-319` |
| POST | `/api/v1/posts` | user | 3 / 10 minutes (post_create, per userId) | Next | `src/app/api/v1/posts/route.ts:223-226` |
| POST | `/api/v1/products` | user | 2 / 1 day (game_submit, per userId) | Next | `src/app/api/v1/products/route.ts:70-73` |
| POST | `/api/v1/upvotes` | user | 100 / 1 day (post_upvote, per userId) | Next | `src/app/api/v1/upvotes/route.ts:176-179` |
| POST | `/api/visky/chat` | public | 10 req/60s per IP (scope visky_chat) + 100/day global (scope visky_global_daily) in KV RATE_LIMIT_KV; FAIL-OPEN on missing KV | Rust | `rust-workers/src/lib.rs:6817 (handler rust-workers/src/lib.rs:7279)` |
| GET | `/api/visky/models` | public | none (KV cache RATE_LIMIT_KV key visky:models, TTL 300s; cache >1h stale rejected) | Rust | `rust-workers/src/lib.rs:6677 (handler rust-workers/src/lib.rs:6582)` |


## 2. Request schema

### 2.1 Query params (GET / HEAD)

#### `GET /api/admin/analytics`

- range: 7d|30d (default 7d)

#### `GET /api/admin/audit-logs`

- page(int>=1,def1)
- limit(int 1..50,def20)

#### `GET /api/admin/games`

- DELETE: id (positive int query param)

#### `GET /api/admin/reports`

- DELETE: id (positive int query param)

#### `GET /api/admin/reviews`

- DELETE: id (positive int query param)

#### `GET /api/bridge/history`

- room_id
- agent_id
- limit

#### `GET /api/bridge/info`

- room

#### `GET /api/bridge/poll`

- room_id
- agent_id
- wait

#### `GET /api/cover-proxy`

- url (target image URL, required)

#### `GET /api/creator/feedback`

- game_id? (positive int filter)

#### `GET /api/games`

- status (default approved; non-approved requires admin)
- category (game|tool|app|art|other|all)
- page/limit in doc but NOT read by Rust (hard LIMIT 50)

#### `GET /api/games/lookup`

- q (query string, <=100 chars, required)

#### `GET /api/og-fallback`

| name | type | required | default | values |
|---|---|---|---|---|
| `title` | string | no | — | — |
| `category` | string | no | — | — |

#### `GET /api/posts`

- GET: page(int>0,def1), limit(int>0,max20,def10), category(discussion|showcase|tutorial|question), game_id(int>0), feed(latest|following)

#### `GET /api/posts/[id]`

- id path param (positive int)
- comment_page(int>0,def1)
- comment_limit(int>0,max20,def20)

#### `GET /api/posts/[id]/comments`

- POST: id path param
- GET: comment_page(int>0,def1), comment_limit(int>0,max20,def20)

#### `GET /api/reviews`

- GET: game_id (positive int, required)

#### `GET /api/rooms/[roomId]/messages`

| name | type | required | default | values |
|---|---|---|---|---|
| `after` | integer (message id) | no | 0 | — |
| `wait` | integer seconds (<=25) | no | 0 | — |

#### `GET /api/rooms/[roomId]/sticky`

| name | type | required | default | values |
|---|---|---|---|---|
| `user_id` | string (bridge id) | yes | — | — |

#### `GET /api/search`

- q (query string, required)

#### `GET /api/upvotes/batch`

- ids (comma-separated positive ints, max 100)

#### `GET /api/users/[id]`

- id path param (^[A-Za-z0-9_-]{1,64}$)

### 2.2 Body mẫu (POST / PATCH / DELETE có body)

#### `DELETE /api/admin/games`

```jsonc
PATCH {"game_id": int, "title"?: string(<=60), "description"?: string(<=160), "play_url"?: string, "cover_image"?: string, "status"?: "approved"|"pending"|"rejected", "is_featured"?: bool, "category"?: string}
```

#### `GET /api/admin/games`

```jsonc
PATCH {"game_id": int, "title"?: string(<=60), "description"?: string(<=160), "play_url"?: string, "cover_image"?: string, "status"?: "approved"|"pending"|"rejected", "is_featured"?: bool, "category"?: string}
```

#### `PATCH /api/admin/games`

```jsonc
PATCH {"game_id": int, "title"?: string(<=60), "description"?: string(<=160), "play_url"?: string, "cover_image"?: string, "status"?: "approved"|"pending"|"rejected", "is_featured"?: bool, "category"?: string}
```

#### `POST /api/admin/report/proposals/[id]`

```jsonc
{"decision": "approved"|"rejected"}
```

#### `PATCH /api/admin/users/[id]/role`

```jsonc
{"role": "user"|"moderator"|"admin"}
```

#### `POST /api/auth/oauth`

Content-Type: `application/json`

| field | type | required | default / ghi chú |
|---|---|---|---|
| `provider` | enum: google | no | google |
| `next` | string (local path) | no | / |

```json
{
  "provider": "google",
  "next": "/dashboard"
}
```

#### `POST /api/auth/sign-in`

Content-Type: `application/json`

| field | type | required | default / ghi chú |
|---|---|---|---|
| `email` | string | yes | — |
| `password` | string | yes | — |

```json
{
  "email": "user@example.com",
  "password": "secret"
}
```

#### `POST /api/auth/sign-up`

Content-Type: `application/json`

| field | type | required | default / ghi chú |
|---|---|---|---|
| `email` | string | yes | — |
| `password` | string | yes | — |

```json
{
  "email": "user@example.com",
  "password": "secret"
}
```

#### `POST /api/bridge/send`

```jsonc
{"room_id":string,"sender_id":string,"receiver_id":string,"content":string,"parent_id"?:string,"end"?:bool}
```

#### `POST /api/creator/feedback/handle`

```jsonc
{"id": int>0, "kind": "review"|"comment"}
```

#### `GET /api/creator/feedback/settings`

```jsonc
PATCH {"notify_feedback": bool}
```

#### `PATCH /api/creator/feedback/settings`

```jsonc
PATCH {"notify_feedback": bool}
```

#### `PATCH /api/creator/games`

Content-Type: `application/json`

| field | type | required | default / ghi chú |
|---|---|---|---|
| `game_id` | integer | yes | — |
| `description` | string (<=5000) | no | — |
| `play_url` | string (http(s) URL) | no | — |

```json
{
  "game_id": 123,
  "description": "Updated description"
}
```

#### `POST /api/creator/send-digest`

Content-Type: `application/json`

| field | type | required | default / ghi chú |
|---|---|---|---|
| `userId` | string (uuid) | no | — |
| `to` | email string | no | — |

```json
{
  "userId": "user-uuid"
}
```

#### `GET /api/games`

```jsonc
POST/PATCH body: {play_url, title?, description?, cover_image?, tags?, category?, slug?} -> Next
```

#### `PATCH /api/games`

Content-Type: `application/json`

| field | type | required | default / ghi chú |
|---|---|---|---|
| `game_id` | integer | yes | — |
| `new_status` | enum: approved|pending|rejected | yes | — |

```json
{
  "game_id": 123,
  "new_status": "approved"
}
```

#### `POST /api/games`

Content-Type: `application/json`

| field | type | required | default / ghi chú |
|---|---|---|---|
| `play_url` | string (http(s) URL) | yes | — |
| `title` | string | no | — |
| `slug` | string | no | — |
| `description` | string | no | — |
| `seo_description` | string | no | — |
| `cover_image` | string | no | — |
| `tags` | string | no | — |
| `category` | string | no | — |
| `platform` | enum: web|mobile|desktop | no | — |

```json
{
  "play_url": "https://example.com/game",
  "title": "My Game",
  "description": "A cool game",
  "tags": "puzzle",
  "category": "game"
}
```

#### `GET /api/posts`

```jsonc
POST {"title": string(3-60), "content": string(1-5000), "category"?: "discussion"|"showcase"|"tutorial"|"question", "game_id"?: int}
```

#### `POST /api/posts`

```jsonc
POST {"title": string(3-60), "content": string(1-5000), "category"?: "discussion"|"showcase"|"tutorial"|"question", "game_id"?: int}
```

#### `GET /api/posts/[id]`

```jsonc
PATCH body {title?,content?,category?,game_id?} -> Next
```

#### `PATCH /api/posts/[id]`

Content-Type: `application/json`

| field | type | required | default / ghi chú |
|---|---|---|---|
| `title` | string | yes | — |
| `content` | string | yes | — |
| `category` | string | no | — |
| `game_id` | integer|null | no | — |

```json
{
  "title": "Updated title",
  "content": "Updated content",
  "category": "tutorial",
  "game_id": null
}
```

#### `GET /api/posts/[id]/comments`

```jsonc
POST {"content": string(1-2000), "parent_id"?: int|string|null}
```

#### `POST /api/posts/[id]/comments`

```jsonc
POST {"content": string(1-2000), "parent_id"?: int|string|null}
```

#### `POST /api/posts/[id]/upvote`

```jsonc
none (body ignored)
```

#### `POST /api/reports`

Content-Type: `application/json`

| field | type | required | default / ghi chú |
|---|---|---|---|
| `game_id` | integer | yes | — |
| `reason` | string (3..500) | yes | — |

```json
{
  "game_id": 42,
  "reason": "Inappropriate content"
}
```

#### `GET /api/reviews`

```jsonc
POST {"game_id": int>0, "rating": int(1-5), "comment"?: string(<=1000 graphemes)}
```

#### `POST /api/reviews`

```jsonc
POST {"game_id": int>0, "rating": int(1-5), "comment"?: string(<=1000 graphemes)}
```

#### `GET /api/rooms`

```jsonc
POST {"name":string,"leader_id":string}
```

#### `POST /api/rooms`

```jsonc
POST {"name":string,"leader_id":string}
```

#### `PATCH /api/rooms/[roomId]`

Content-Type: `application/json`

| field | type | required | default / ghi chú |
|---|---|---|---|
| `leader_id` | string|null (registered agent) | yes | — |

```json
{
  "leader_id": "agent-1"
}
```

#### `POST /api/rooms/[roomId]/agents`

Content-Type: `application/json`

| field | type | required | default / ghi chú |
|---|---|---|---|
| `agent_id` | string (bridge id) | yes | — |
| `display_name` | string | no | — |

```json
{
  "agent_id": "agent-2",
  "display_name": "Researcher"
}
```

#### `PATCH /api/rooms/[roomId]/agents/[agentId]/rename`

Content-Type: `application/json`

| field | type | required | default / ghi chú |
|---|---|---|---|
| `display_name` | string (1..80) | yes | — |

```json
{
  "display_name": "New Name"
}
```

#### `DELETE /api/rooms/[roomId]/sticky`

Content-Type: `application/json`

| field | type | required | default / ghi chú |
|---|---|---|---|
| `user_id` | string (bridge id) | yes | — |
| `agent_id` | string | yes | — |

```json
{
  "user_id": "user-1",
  "agent_id": "agent-2"
}
```

#### `POST /api/rooms/[roomId]/sticky`

Content-Type: `application/json`

| field | type | required | default / ghi chú |
|---|---|---|---|
| `user_id` | string (bridge id) | yes | — |
| `agent_id` | string (registered agent) | yes | — |

```json
{
  "user_id": "user-1",
  "agent_id": "agent-2"
}
```

#### `POST /api/submit/analyze`

```jsonc
{"url": string (public URL)}
```

#### `POST /api/upload`

Content-Type: `multipart/form-data`

| field | type | required | default / ghi chú |
|---|---|---|---|
| `file` | File (image/jpeg|png|webp, <=2MB) | yes | — |

```json
{
  "file": "<binary JPEG/PNG/WebP>"
}
```

#### `POST /api/upvotes`

```jsonc
POST {"game_id": int>0}
```

#### `POST /api/users/[id]/follow`

```jsonc
none
```

#### `PATCH /api/users/me`

Content-Type: `application/json`

| field | type | required | default / ghi chú |
|---|---|---|---|
| `display_name` | string | no | — |
| `bio` | string | no | — |
| `avatar_url` | string (http(s) URL) | no | — |

```json
{
  "display_name": "Culi",
  "bio": "Frontend dev",
  "avatar_url": "https://img.example/me.png"
}
```

#### `POST /api/v1/comments`

Content-Type: `application/json`

| field | type | required | default / ghi chú |
|---|---|---|---|
| `post_id` | integer | yes | — |
| `content` | string (1..1000) | yes | — |
| `parent_id` | integer | no | — |
| `via_agent` | string (<=40) | no | — |

```json
{
  "post_id": 42,
  "content": "Nice!",
  "parent_id": null,
  "via_agent": "ChatGPT"
}
```

#### `POST /api/v1/keys`

Content-Type: `application/json`

| field | type | required | default / ghi chú |
|---|---|---|---|
| `name` | string (<=60) | no | — |
| `scopes` | string[] (post:write|product:submit|comment:write|upvote:write) | yes | — |
| `expires_in_days` | integer 1..365 | no | — |

```json
{
  "name": "My agent",
  "scopes": [
    "post:write",
    "comment:write"
  ],
  "expires_in_days": 30
}
```

#### `POST /api/v1/posts`

Content-Type: `application/json`

| field | type | required | default / ghi chú |
|---|---|---|---|
| `title` | string (3..200) | yes | — |
| `content` | string (1..5000) | yes | — |
| `category` | enum: discussion|showcase|tutorial|question | no | — |
| `game_id` | integer | no | — |
| `via_agent` | string (<=40) | no | — |

```json
{
  "title": "Agent post",
  "content": "Written by an agent",
  "via_agent": "ChatGPT"
}
```

#### `POST /api/v1/products`

Content-Type: `application/json`

| field | type | required | default / ghi chú |
|---|---|---|---|
| `play_url` | string (http(s) URL) | yes | — |
| `slug` | string | no | — |
| `title` | string | no | — |
| `description` | string | no | — |
| `cover_image` | string | no | — |
| `tags` | string | no | — |
| `category` | enum: game|tool|app|art|other|auto | no | — |

```json
{
  "play_url": "https://example.com/tool",
  "title": "My Tool",
  "category": "tool"
}
```

#### `POST /api/v1/upvotes`

Content-Type: `application/json`

| field | type | required | default / ghi chú |
|---|---|---|---|
| `post_id` | integer | yes | — |
| `via_agent` | string (<=40) | no | — |

```json
{
  "post_id": 42,
  "via_agent": "ChatGPT"
}
```

#### `POST /api/visky/chat`

```jsonc
{"message": string(1-2000 chars) required, "model": string? , "sessionId": string?, "history": [{role:string,content:string}]? (max 40)}
```

## 3. Response schema

### 3.1 Envelope & kiểu dữ liệu

| Dạng | Ví dụ | Khi dùng |
|---|---|---|
| Bare object | `{"games": Game[]}` | Một số GET của Rust (`/api/games`, `/api/games/lookup`) |
| `data` | `{"data": {posts: Post[], page: number, has_more: boolean}}` | Hầu hết read API |
| `success` | `{"success": true, ...}` | Hầu hết write API |
| SSE | `data: {"type":"chunk","content":"..."}\n\n` | `POST /api/visky/chat` |

Field type legend: `string` · `number` (int/float) · `boolean` · `null` · `T[]` · `T | null` · `object`.

Lỗi (biến thể tuỳ layer; không phải endpoint nào cũng trả mọi field):

```jsonc
// 400 — validate
{ "success": false, "error": "VALIDATION", "message": "title quá ngắn" }
// hoặc { "error": "Invalid request" }

// 401 — chưa đăng nhập / token sai
{ "success": false, "error": "UNAUTHORIZED" }   // Next
{ "error": "Chưa đăng nhập" }                    // Rust

// 403 — sai role / sai scope / không phải chủ phòng
{ "success": false, "error": "FORBIDDEN" }
{ "error": "MISSING_SCOPE", "scope": "product:submit" }   // API key v1

// 429 — app KV hoặc edge rule (thêm Retry-After nếu do Cloudflare)
{ "success": false, "error": "RATE_LIMITED", "retry_after_seconds": 60, "window_count": 31, "window_limit": 30 }

// 500 / 503
{ "success": false, "error": "INTERNAL_ERROR" }
{ "error": "Feature chưa sẵn sàng", "migration_required": true }
```

| Status | Ý nghĩa | Field chính |
|---|---|---|
| 400 | Input/body/query sai | `error: string`, `message?: string` |
| 401 | Thiếu/sai cookie hoặc Bearer | `error: string` |
| 403 | Đúng user nhưng sai role/scope/ownership | `error: string` |
| 404 | Không tìm thấy resource (cố tình dùng cho cả "không phải chủ phòng") | `error: string` |
| 413 | File upload vượt 2MB | `error: string` |
| 429 | Hết quota app KV hoặc edge WAF | `error: string`, `retry_after_seconds?: number`, `window_count?: number`, `window_limit?: number` |
| 500 | Lỗi máy chủ / upstream | `error: string` |
| 503 | Migration/binding/upstream chưa có | `error: string`, `migration_required?: boolean` |

### 3.2 Success shape & error statuses theo endpoint

| Method + Path | HTTP | Success shape (kiểu) | Error statuses |
|---|---|---|---|
| `ANY /*catchall` | 200 | `Whatever Next.js returns, forwarded verbatim with headers stripped (host/x-forwarded-*/content-length)` | 5, 0, 2,  , {, e, r, r, o, r, :, ', K, h, ô, n, g,  , t, h, ể,  , k, ế, t,  , n, ố, i,  , t, ớ, i,  , m, á, y,  , c, h, ủ,  , n, ộ, i,  , b, ộ, ', },  , i, f,  , V, I, O, N, S, K, Y, _, W, E, B,  , S, e, r, v, i, c, e,  , B, i, n, d, i, n, g,  , m, i, s, s, i, n, g, /, f, a, i, l, s |
| `GET /api/admin/analytics` | 200 | `200 {range:{days:int,fromDay:string,toDay:string,generatedAt:string},content:{bySource:object,total:number},api:{topScopes:object,totalRequests:number,errorRequests:number,errorRate:number},ai:{average:number,total:number,trend:object},search:{byOutcome:object,zeroTotal:number,zeroByLocale:object},reliability:{errorsByCode:object,rateLimitsByScope:object,revokedKeys:number,bypassedTotal:number,bypassedByRole:objec...` | 401/403 {error:string}, 500 {error:'Không tải được số liệu analytics.'} on failure, Empty arrays if daily_metrics table missing (migration 022) |
| `GET /api/admin/audit-logs` | 200 | `200 {logs: [{id:int,admin_id:string,admin_email:string,action:string,target_type:string,target_id:string,metadata:string,created_at:string}], pagination:{page:int,limit:int,total:int,totalPages:int}}` | 401/403 {error:string}, 500 {error:'Không tải được audit log'} if URL parse fails |
| `DELETE /api/admin/games` | 200 | `GET 200 {games: Game[]} (whitelisted columns); PATCH 200 {success:true,game: object/null}; DELETE 200 {success:true,deletedId:int}` | 400 {error:string} invalid id/fields, 401/403 {error:string} from verify_admin/verify_moderator, 404 {error:'Không tìm thấy game'} on DELETE, 500 {error:string} on batch failure |
| `GET /api/admin/games` | 200 | `GET 200 {games: Game[]} (whitelisted columns); PATCH 200 {success:true,game: object/null}; DELETE 200 {success:true,deletedId:int}` | 400 {error:string} invalid id/fields, 401/403 {error:string} from verify_admin/verify_moderator, 404 {error:'Không tìm thấy game'} on DELETE, 500 {error:string} on batch failure |
| `PATCH /api/admin/games` | 200 | `GET 200 {games: Game[]} (whitelisted columns); PATCH 200 {success:true,game: object/null}; DELETE 200 {success:true,deletedId:int}` | 400 {error:string} invalid id/fields, 401/403 {error:string} from verify_admin/verify_moderator, 404 {error:'Không tìm thấy game'} on DELETE, 500 {error:string} on batch failure |
| `GET /api/admin/report` | 200 | `200 {report: {id:number,periodStart:string,periodEnd:string,compareStart:string,compareEnd:string,generatedAt:string,markdown:string,payload:object}/null, proposals: [{id:string,kind:string,title:string,rationale:string,severity:string,state:string,periodStart:string,periodEnd:string,context:object,decidedAt:string/null,decidedBy:string/null,createdAt:string}]}` | 401/403 {error:string}, 503 MIGRATION_REQUIRED if weekly_reports missing, 500 {error:'Không tải được báo cáo tuần.'} |
| `POST /api/admin/report/proposals/[id]` | 200 | `200 {success:true,id:string,decision:string}` | 400 {error:...} missing id / invalid JSON / bad decision, 401/403 {error:string}, 409 {error:'Đề xuất không tồn tại hoặc đã được quyết định.'} (already decided / missing table), 500 {error:'Không lưu được quyết định.'} |
| `DELETE /api/admin/reports` | 200 | `GET 200 {reports: [{id:int,game_id:int,user_id:string,reason:string,created_at:string,game_title:string/null,game_slug:string/null}]} (<=200); DELETE 200 {success:true,deletedId:int}` | 400 {error:'id không hợp lệ'}, 401/403 {error:string}, 404 {error:'Không tìm thấy báo cáo'}, 500 {error:'Không xóa được báo cáo'} |
| `GET /api/admin/reports` | 200 | `GET 200 {reports: [{id:int,game_id:int,user_id:string,reason:string,created_at:string,game_title:string/null,game_slug:string/null}]} (<=200); DELETE 200 {success:true,deletedId:int}` | 400 {error:'id không hợp lệ'}, 401/403 {error:string}, 404 {error:'Không tìm thấy báo cáo'}, 500 {error:'Không xóa được báo cáo'} |
| `DELETE /api/admin/reviews` | 200 | `GET 200 {reviews: [{id:int,game_id:int,user_name:string,rating:int,comment:string/null,created_at:string,game_title:string/null,game_slug:string/null}]} (is_current=1, <=200); DELETE 200 {success:true,deletedId:int}` | 400 {error:'id không hợp lệ'}, 401/403 {error:string}, 404 {error:'Không tìm thấy đánh giá'}, 500 {error:'Không xóa được đánh giá'} |
| `GET /api/admin/reviews` | 200 | `GET 200 {reviews: [{id:int,game_id:int,user_name:string,rating:int,comment:string/null,created_at:string,game_title:string/null,game_slug:string/null}]} (is_current=1, <=200); DELETE 200 {success:true,deletedId:int}` | 400 {error:'id không hợp lệ'}, 401/403 {error:string}, 404 {error:'Không tìm thấy đánh giá'}, 500 {error:'Không xóa được đánh giá'} |
| `GET /api/admin/stats` | 200 | `200 {total_games:int,total_pending:int,total_users:int,total_reviews:int,total_reports:int}` | 401/403 {error:string} from verify_moderator |
| `GET /api/admin/system-health` | 200 | `200 {source:string,d1:object,kv:object,workers:{requests:number,errors:number,errorRate:number,exceededResources:number,cpuTimeP50Ms:number/null,cpuTimeP99Ms:number/null,cpuLimitMs:number,cpuUsagePercent:number/null,scripts:Array,estimated:bool},alerts:Array,window:{hours:int,from:string,to:string},generatedAt:string,cached:bool,cacheAgeSeconds:number}; Cache-Control: private, no-store` | 401/403 {error:string} |
| `GET /api/admin/users` | 200 | `200 {users: [{user_id:string,user_name:string,games_submitted:int,reviews_posted:int,last_activity:string,role:string}]} (<=300)` | 401/403 {error:string} from verify_admin, 503 MIGRATION_REQUIRED if users table missing, 500 {error:'Không tải được danh sách người dùng'} |
| `PATCH /api/admin/users/[id]/role` | 200 | `200 {success:true,data:{id:string,role:string}}` | 400 {error:'ID người dùng không hợp lệ'|'Body không phải JSON hợp lệ'|'role phải là user, moderator hoặc admin'|'Bạn không thể tự thay đổi quyền của chính mình'}, 401/403 {error:string}, 503 MIGRATION_REQUIRED (roles 015 / users 009), 500 {error:'Không cập nhật được quyền'} |
| `POST /api/auth/oauth` | 200 | `{"url":"string (Supabase authorize URL)"}` | 400,500 |
| `GET /api/auth/session` | 200 | `{"user":"{id: string, email: string} / null"}` | — |
| `POST /api/auth/sign-in` | 200 | `{"user":"{id: string/null, email: string/null}"}` | 400,401 |
| `POST /api/auth/sign-out` | 200 | `{"success":"true (boolean)"}` | — |
| `POST /api/auth/sign-up` | 200 | `{"user":"{id: string/null, email: string/null}","needsConfirmation":"boolean"}` | 400 |
| `GET /api/bridge/history` | 200 | `Next.js history JSON — forwarded` | N, e, x, t, ., j, s,  , e, r, r, o, r, s |
| `GET /api/bridge/info` | 200 | `Next.js self-discovery JSON — forwarded` | N, e, x, t, ., j, s,  , e, r, r, o, r, s |
| `GET /api/bridge/poll` | 200 | `Next.js poll JSON (long-poll) — forwarded` | N, e, x, t, ., j, s,  , e, r, r, o, r, s |
| `POST /api/bridge/send` | 200 | `Next.js send result — forwarded` | N, e, x, t, ., j, s,  , e, r, r, o, r, s |
| `GET /api/cover-proxy` | 200 | `200 raw image bytes; Content-Type: string(image/*); Cache-Control: public, max-age=86400, stale-while-revalidate=604800; X-Content-Type-Options: nosniff` | 302 redirect to /images/placeholder-cover.svg on: missing/invalid url, non-http(s) scheme, private/SSRF host (localhost/127.*/10.*/172.16-31.*/192.168.*/169.254.*/::1/*.local/*.internal), upstream >=400, non-image Content-Type, bytes > 5MiB |
| `GET /api/creator/feedback` | 200 | `200 {success:true,items:[{kind:"review"/"comment",id:int,game_id:int,game_title:string,game_slug:string,user_name:string,rating:int/null,content:string,created_at:string}],total:int,digest:{feedback_count:int,warning_count:int,warning_message:string/null,game_title:string/null,game_slug:string/null}} (no-store, max 50 items)` | 401 {error:'Vui lòng đăng nhập'}, 500 on DB error |
| `POST /api/creator/feedback/handle` | 200 | `200 {success:true} (no-store)` | 401 {error:...}, 400 {error:'Body không phải JSON hợp lệ'|'id không hợp lệ'|'kind phải là review hoặc comment'}, 404 {error:'Không tìm thấy góp ý'}, 403 {error:'Không có quyền xử lý góp ý này'} |
| `GET /api/creator/feedback/settings` | 200 | `200 {success:true,notify_feedback:bool} (no-store)` | 401 {error:...}, 400 {error:...} invalid body or non-bool notify_feedback |
| `PATCH /api/creator/feedback/settings` | 200 | `200 {success:true,notify_feedback:bool} (no-store)` | 401 {error:...}, 400 {error:...} invalid body or non-bool notify_feedback |
| `GET /api/creator/games` | 200 | `{"games":"CreatorGame[] (incl. ai_score, ai_review, daily_audit_count, average_rating, review_count)"}` | 401,500 |
| `PATCH /api/creator/games` | 200 | `{"success":"true","re_review":"boolean"}` | 400,401,403,404,500 |
| `POST /api/creator/games/[id]/re-audit` | 200 | `{"success":"true","re_audit_started":"true","daily_audit_count":"number","max_daily_audits":"3"}` | 400,401,403,404,429,500 |
| `POST /api/creator/send-digest` | 200 | `{"success":"true, ...result"}` | 400,401,502,500 |
| `GET /api/games` | 200 | `GET 200 {games: [{id:int,title:string,slug:string,description:string/null,cover_image:string/null,play_url:string,creator:string/null,tags:string/null,plays:int,rating:number,created_at:string,status:string,category:string/null,is_featured:int,upvote_count:int,is_learning:bool}]} (<=50); Cache-Control: no-store` | 400 {error:string} invalid category or invalid status auth, 401/403 from verify_admin when status!=approved |
| `PATCH /api/games` | 200 | `{"success":"true"}` | 400,401,403 |
| `POST /api/games` | 201 | `{"success":"true","gameId":"number/null","status":"'pending'"}` | 400,401,409,429,500 |
| `DELETE /api/games/[id]` | 200 | `{"success":"true","data":"{id: number, mode: string, status: string/null}"}` | 400,401,403,404,500 |
| `POST /api/games/[id]/play` | 200 | `{"success":"true"}` | 400,404,500 |
| `GET /api/games/lookup` | 200 | `200 {games: [{id:int,title:string,slug:string,cover_image:string/null,rating:number/null,category:string/null}]} (<=20, status=approved only); Cache-Control: public, max-age=30, s-maxage=30` | 429 {error:string,games:[]} when quota exceeded OR KV binding missing (fail-closed) |
| `DELETE /api/notifications` | 200 | `GET 200 {success:true,notifications:[{id:int,user_id:string,type:string,message:string,link:string,is_read:bool,created_at:string}],unread_count:int,limit:int(20)}; DELETE 200 {success:true,deleted:int,unread_count:int}` | 401 (auth.status) {error:string} GET / {success:false,error:string} DELETE, 500 {error:'Không tải được thông báo'}, 200 with migration_required:true + empty list if notifications table missing |
| `GET /api/notifications` | 200 | `GET 200 {success:true,notifications:[{id:int,user_id:string,type:string,message:string,link:string,is_read:bool,created_at:string}],unread_count:int,limit:int(20)}; DELETE 200 {success:true,deleted:int,unread_count:int}` | 401 (auth.status) {error:string} GET / {success:false,error:string} DELETE, 500 {error:'Không tải được thông báo'}, 200 with migration_required:true + empty list if notifications table missing |
| `PATCH /api/notifications/read` | 200 | `200 {success:true,updated:int,unread_count:int}` | 401 {error:string} (NO success field on this route's errors), 500 {error:'Không cập nhật được thông báo'}, 200 migration_required:true on missing table |
| `GET /api/og-fallback` | 200 | `image/svg+xml SVG placeholder (not JSON)` | — |
| `GET /api/ping` | 200 | `200 {status: string("ok"), runtime: string("rust-wasm")}` | 404 if unknown path |
| `GET /api/posts` | 200 | `POST 201 {success:bool(true),data:{id:int}}; GET 200 {success:true,data:{posts: Post[],page:int,limit:int,has_more:bool,feed:string,following_count:int/null}} (no-store)` | 401 {success:false,error:...}, 400 on invalid body/game_id/title/content, 429 {success:false,error:string,window_count:int,window_limit:int}, 503 if posts table missing (migration 007), 500 generic |
| `POST /api/posts` | 200 | `POST 201 {success:bool(true),data:{id:int}}; GET 200 {success:true,data:{posts: Post[],page:int,limit:int,has_more:bool,feed:string,following_count:int/null}} (no-store)` | 401 {success:false,error:...}, 400 on invalid body/game_id/title/content, 429 {success:false,error:string,window_count:int,window_limit:int}, 503 if posts table missing (migration 007), 500 generic |
| `DELETE /api/posts/[id]` | 200 | `{"success":"true","data":"{id: number}"}` | 400,401,403,404,500,503 |
| `GET /api/posts/[id]` | 200 | `GET 200 {success:true,data:{post: Post,comments: Comment[],comment_page:int,comment_limit:int,has_more_comments:bool}} (no-store); post includes locale:"vi"` | 400 {success:false,error:'ID bài viết không hợp lệ'}, 404 {success:false,error:'Không tìm thấy bài viết'} |
| `PATCH /api/posts/[id]` | 200 | `{"success":"true","data":"{post: Post}"}` | 400,401,403,404,500,503 |
| `GET /api/posts/[id]/comments` | 200 | `POST 201 {success:true,data:{comment:{id:int,post_id:int,user_id:string,user_name:string,content:string,created_at:string,parent_id:int/null}}}; GET 200 {success:true,data:{comments: Comment[],comment_page:int,comment_limit:int,has_more_comments:bool}} (no-store)` | 400 {success:false,error:...} bad id/content/parent_id/parent comment, 401 {success:false,error:...}, 404 {success:false,error:'Không tìm thấy bài viết'}, 429 {success:false,error:string,window_count:int,window_limit:int}, 503 missing post_comments table, 500 generic |
| `POST /api/posts/[id]/comments` | 200 | `POST 201 {success:true,data:{comment:{id:int,post_id:int,user_id:string,user_name:string,content:string,created_at:string,parent_id:int/null}}}; GET 200 {success:true,data:{comments: Comment[],comment_page:int,comment_limit:int,has_more_comments:bool}} (no-store)` | 400 {success:false,error:...} bad id/content/parent_id/parent comment, 401 {success:false,error:...}, 404 {success:false,error:'Không tìm thấy bài viết'}, 429 {success:false,error:string,window_count:int,window_limit:int}, 503 missing post_comments table, 500 generic |
| `DELETE /api/posts/[id]/comments/[commentId]` | 200 | `{"success":"true","data":"{id: number}"}` | 400,401,403,404,500,503 |
| `PATCH /api/posts/[id]/comments/[commentId]` | 200 | `{"success":"true","data":"{id: number, is_pinned: boolean}"}` | 400,401,403,404,500,503 |
| `POST /api/posts/[id]/upvote` | 200 | `200 {success:bool(true),upvoted:bool,upvote_count:int}` | 400 {success:false,error:'ID bài viết không hợp lệ'}, 401 {success:false,error:...}, 404 {success:false,error:'Không tìm thấy bài viết'}, 429 {success:false,error:string,daily_count:int,max_daily_upvotes:int}, 503 missing post_upvotes table (migration 008b), 500 generic |
| `POST /api/reports` | 201 | `{"success":"true","reportId":"number/null"}` | 400,401,404,429,500 |
| `GET /api/reviews` | 200 | `POST 201 {success:true,reviewId:int,rating:int,average_rating:number}; GET 200 {reviews: [{id:int,game_id:int,user_name:string,rating:int,comment:string/null,created_at:string,is_current:int,role:string,is_creator:bool}], average_rating:number/null, total_reviews:int} (no-store)` | 401 {error:...}, 400 {error:...} invalid game_id/rating, 404 {error:'Không tìm thấy sản phẩm'}, 429 {error:string,window_count:int,window_limit:int}, 503 missing reviews table, 500 generic |
| `POST /api/reviews` | 200 | `POST 201 {success:true,reviewId:int,rating:int,average_rating:number}; GET 200 {reviews: [{id:int,game_id:int,user_name:string,rating:int,comment:string/null,created_at:string,is_current:int,role:string,is_creator:bool}], average_rating:number/null, total_reviews:int} (no-store)` | 401 {error:...}, 400 {error:...} invalid game_id/rating, 404 {error:'Không tìm thấy sản phẩm'}, 429 {error:string,window_count:int,window_limit:int}, 503 missing reviews table, 500 generic |
| `GET /api/rooms` | 200 | `Next.js {success:true,rooms:Room[]} / {success:true,roomId,room_secret} — forwarded` | N, e, x, t, ., j, s,  , e, r, r, o, r, s |
| `POST /api/rooms` | 200 | `Next.js {success:true,rooms:Room[]} / {success:true,roomId,room_secret} — forwarded` | N, e, x, t, ., j, s,  , e, r, r, o, r, s |
| `DELETE /api/rooms/[roomId]` | 200 | `{"success":"true"}` | 401,404 |
| `PATCH /api/rooms/[roomId]` | 200 | `{"success":"true","leader_id":"string/null"}` | 400,401,404 |
| `GET /api/rooms/[roomId]/agents` | 200 | `{"success":"true","agents":"{agent_id: string, display_name: string}[]"}` | 400,401,404 |
| `POST /api/rooms/[roomId]/agents` | 201 | `{"success":"true","room_id":"string","agent_id":"string","display_name":"string"}` | 400,401,404,409 |
| `DELETE /api/rooms/[roomId]/agents/[agentId]` | 200 | `{"success":"true","agent_id":"string"}` | 400,401,404 |
| `PATCH /api/rooms/[roomId]/agents/[agentId]/rename` | 200 | `{"success":"true","agent_id":"string","display_name":"string"}` | 400,401,404,409 |
| `GET /api/rooms/[roomId]/messages` | 200 | `{"success":"true","messages":"Array<Record<string,unknown>>"}` | 401,404 |
| `GET /api/rooms/[roomId]/presence` | 200 | `{"success":"true","count":"number","agents":"string[]"}` | 401,404 |
| `POST /api/rooms/[roomId]/rotate` | 200 | `{"success":"true","room_secret":"string (new secret)"}` | 401,404 |
| `DELETE /api/rooms/[roomId]/sticky` | 200 | `{"success":"true","agent_id":"string"}` | 400,401,404 |
| `GET /api/rooms/[roomId]/sticky` | 200 | `{"success":"true","user_id":"string","agents":"string[]"}` | 400,401,404 |
| `POST /api/rooms/[roomId]/sticky` | 201 | `{"success":"true","agent_id":"string"}` | 400,401,404 |
| `GET /api/search` | 200 | `200 {games: [{id:int,title:string,slug:string,description:string,cover_image:string,category:string,plays:int,rating:number,created_at:string}], posts: [{id:int,title:string,content_preview:string,user_name:string,created_at:string,comment_count:int}]} (<=10 each); Cache-Control: public, max-age=30, s-maxage=30, stale-while-revalidate=60` | 429 {error:string,retry_after_seconds:int(60)} on quota exceeded, 429 {error:string} if KV binding missing (fail-closed), 500 {error:'Không tìm kiếm được'} on DB error |
| `POST /api/submit/analyze` | 200 | `Next.js {success:true,draft} — forwarded verbatim` | N, e, x, t, ., j, s,  , e, r, r, o, r, s,  , (, I, N, V, A, L, I, D, _, U, R, L, /, B, L, O, C, K, E, D, _, T, A, R, G, E, T, /, F, E, T, C, H, _, F, A, I, L, E, D, /, R, A, T, E, _, L, I, M, I, T, E, D, ) |
| `POST /api/upload` | 201 | `{"success":"true","url":"string"}` | 400,401,413,429,500,503 |
| `GET /api/upvotes` | 200 | `{"game_ids":"number[]"}` | 401,500 |
| `POST /api/upvotes` | 200 | `POST 200 {success:bool(true),upvoted:bool,upvote_count:int}; GET forwarded {game_ids: number[]}` | 401 {error:'Vui lòng đăng nhập để thả tim'}, 400 {error:'game_id không hợp lệ'}, 404 {error:'Không tìm thấy sản phẩm'}, 429 {error:string,daily_count:int,max_daily_upvotes:int}, 503 if upvotes table missing (migration 006), 500 generic DB error |
| `GET /api/upvotes/batch` | 200 | `200 {string(gameId): int(count)} map — every requested id present, missing = 0; Cache-Control: public, max-age=30, s-maxage=60, stale-while-revalidate=120` | No explicit error status: always 200 {} on bad/missing ids, missing DB binding, or query error (fail-open) |
| `GET /api/users/[id]` | 200 | `200 {success:true,user:{id:string,display_name:string,bio:string,avatar_url:string,created_at:string,updated_at:string,role:string,is_creator:bool,follower_count:int,following_count:int,is_following:bool}}` | 400 {error:'ID người dùng không hợp lệ'}, 404 {error:'Không tìm thấy người dùng'}, 503 {error:USERS_MIGRATION_HINT,code:'MIGRATION_REQUIRED'}, 200 with follows_migration_required:true + follows_migration_hint if follows table missing, 500 {error:'Không tải được hồ sơ'} |
| `POST /api/users/[id]/follow` | 200 | `200 {success:true,is_following:bool,follower_count:int,following_count:int}` | 400 {success:false,error:...} bad id or self-follow, 401 (auth.status) {success:false,error:string}, 404 {success:false,error:'Không tìm thấy người dùng'}, 429 {success:false,error:string,daily_count:int,max_daily_follows:int}, 503 MIGRATION_REQUIRED (follows/users tables), 500 generic |
| `DELETE /api/users/me` | 200 | `{"success":"true","data":"{deleted_games: number, affected_posts: number, supabase_user_deleted: boolean}","warning?":"string"}` | 401,500 |
| `GET /api/users/me` | 200 | `200 {success:true,exists:bool,user:{id:string,email:string,display_name:string,bio:string,avatar_url:string,created_at:string,updated_at:string,role:string,is_creator:bool}}` | 401 {error:string}, 503 {error:MIGRATION_REQUIRED hint,code:'MIGRATION_REQUIRED'} if users table missing, 500 {error:'Không tải được hồ sơ'} |
| `PATCH /api/users/me` | 200 | `{"success":"true","user":"User profile","exists":"true"}` | 400,401,500,503 |
| `POST /api/users/sync` | 200 | `Next.js response (forwarded verbatim)` | Next.js errors (e.g. 401/403/500) forwarded |
| `POST /api/v1/comments` | 201 | `{"success":"true","data":"{id, post_id, parent_id, url, via_agent}"}` | 400,401,403,404,429,500,503 |
| `DELETE /api/v1/keys` | 200 | `{"success":"true","data":"{id: string, revoked: true}"}` | 400,401,403,404,500 |
| `GET /api/v1/keys` | 200 | `{"success":"true","data":"{keys: ApiKey[], scopes: string[]}"}` | 401,500 |
| `POST /api/v1/keys` | 201 | `{"success":"true","data":"{id, name, scopes, key_prefix, expires_at, created_at}","secret":"string (plaintext, shown ONCE)","warning":"string"}` | 400,401,429,500 |
| `GET /api/v1/openapi.json` | 200 | `OpenAPI 3.1 JSON document (static)` | — |
| `POST /api/v1/posts` | 201 | `{"success":"true","data":"{id: number/null, url: string/null, via_agent: string/null}"}` | 400,401,403,429,500,503 |
| `POST /api/v1/products` | 201 | `{"success":"true","data":"{id, slug, status:'pending', url}"}` | 400,401,403,409,429,500 |
| `POST /api/v1/upvotes` | 200 | `{"success":"true","data":"{post_id: number, upvoted: true, upvote_count: number, via_agent: string/null}"}` | 400,401,403,404,429,500 |
| `POST /api/visky/chat` | 200 | `200 SSE text/event-stream: data: {"type":"start"}\n\n then token/error/done events; headers Cache-Control: no-cache, no-transform; Connection: keep-alive; X-Accel-Buffering: no` | 400 {success:false,error:{code,message}} codes INVALID_BODY, EMPTY_MESSAGE, MESSAGE_TOO_LONG, 429 RATE_LIMITED (IP 10/60s and global 100/day, fail-open on KV error), 500 VISKY_GATEWAY_MISCONFIGURED (bad VISKY_ZEROCLAW_URL), VISKY_NOT_CONFIGURED (missing VISKY_ZEROCLAW_TOKEN) |
| `GET /api/visky/models` | 200 | `200 {success: bool(true), data: {models: []}}; Cache-Control: "public, max-age=0, s-maxage=300, stale-while-revalidate=60"` | 503 {success:false,error:{code:string,message:string}} codes: VISKY_NOT_CONFIGURED (missing VISKY_ZEROCLAW_URL), VISKY_UPSTREAM_ERROR, VISKY_UPSTREAM_TIMEOUT |


## 4. Auth flow

### 4.1 Cách Culi gọi Supabase Auth

```ts
// 1) Đăng nhập email/password
await fetch('/api/auth/sign-in', { method:'POST', credentials:'same-origin',
  headers:{'Content-Type':'application/json'}, body: JSON.stringify({email,password}) });
// 2) Hoặc Google OAuth
const { url } = await (await fetch('/api/auth/oauth', { method:'POST',
  headers:{'Content-Type':'application/json'}, body:'{"provider":"google","next":"/vi/channels"}' })).json();
location.href = url;   // Supabase -> GET /auth/callback?code=... -> set HttpOnly cookie -> redirect next
// 3) Biết trạng thái đăng nhập
const { user } = await (await fetch('/api/auth/session', {credentials:'same-origin'})).json();
// user = { id: string, email: string } | null  (KHÔNG chứa access_token)
```

- **provider:** Supabase GoTrue is the identity provider; project URL https://lzrypnlbsrwuifjkjdyc.supabase.co, anon/publishable key NEXT_PUBLIC_SUPABASE_ANON_KEY (public, shipped in client). `/root/vionsky/wrangler.jsonc:30-32` `/root/vionsky/rust-workers/wrangler.toml:61-63`
- **server_verify_endpoint:** Both Next and Rust verify tokens online via GET {SUPABASE_URL}/auth/v1/user with headers `Authorization: Bearer <token>` and `apikey: <anon key>`, timeout 5s (Next AbortSignal.timeout(5000); Rust Promise.race/Delay 5000ms). On network/timeout -> 401 fail-closed; 5xx -> 500. No JWT secret or self-verify. `/root/vionsky/src/lib/auth.ts:32-56` `/root/vionsky/rust-workers/src/auth.rs:394-495` `/root/vionsky/API_CONTRACT.md:38-39`
- **auth_endpoints:** POST /api/auth/sign-in {email,password} -> supabase.auth.signInWithPassword; sets HttpOnly cookie; 400/401 {error}. `/root/vionsky/src/app/api/auth/sign-in/route.ts:4-19` `/root/vionsky/API_CONTRACT.md:80`
- **auth_endpoints:** POST /api/auth/sign-up {email,password} -> supabase.auth.signUp; returns needsConfirmation = !data.session. `/root/vionsky/src/app/api/auth/sign-up/route.ts:4-22` `/root/vionsky/API_CONTRACT.md:81`
- **auth_endpoints:** POST /api/auth/oauth {provider?:google,next?} -> signInWithOAuth(google) with redirectTo = `${SITE_URL}/auth/callback?next=...`; SITE_URL = https://vionsky.com from vars (request.url host is 'undefined' on OpenNext). Non-google -> 400. `/root/vionsky/src/app/api/auth/oauth/route.ts:21-53` `/root/vionsky/wrangler.jsonc:35` `/root/vionsky/API_CONTRACT.md:79`
- **auth_endpoints:** GET /auth/callback?code=&next= -> exchangeCodeForSession(code) -> sets HttpOnly cookie, 302 to `next` (safe local path only) or /?auth_error=oauth. `/root/vionsky/src/app/auth/callback/route.ts:5-23` `/root/vionsky/API_CONTRACT.md:78`
- **auth_endpoints:** GET /api/auth/session -> supabase.auth.getUser(), returns {user:{id,email}|null} only; NEVER returns the token. `/root/vionsky/src/app/api/auth/session/route.ts:4-12` `/root/vionsky/API_CONTRACT.md:83`
- **auth_endpoints:** POST /api/auth/sign-out -> supabase.auth.signOut() clears cookie -> {success:true}. `/root/vionsky/src/app/api/auth/sign-out/route.ts:4-10` `/root/vionsky/API_CONTRACT.md:82`
- **auth_endpoints:** Middleware creates @supabase/ssr server client and calls auth.getClaims() (verifies JWT signature, unlike getSession()) to refresh the session cookie on every request; cookie options httpOnly/secure/sameSite=lax/path=/; for /api/* forces Cache-Control no-store, removes CORS header, adds Vary: Cookie. `/root/vionsky/src/utils/supabase/proxy.ts:4-49`
- **cookie / name:** Cookie name `sb-<project-ref>-auth-token`; project ref = first hostname label of SUPABASE_URL => production `sb-lzrypnlbsrwuifjkjdyc-auth-token`. `/root/vionsky/src/lib/auth-cookie.ts:88-97` `/root/vionsky/rust-workers/src/auth.rs:141-163,618-619` `/root/vionsky/wrangler.jsonc:31`
- **cookie / format:** @supabase/ssr >= 0.12 writes `base64-<base64url(JSON session)>`; decoder strips `base64-`, base64url-decodes to UTF-8, JSON.parses, reads `access_token`. Legacy plain JSON / URI-encoded / bare base64url also accepted for old clients. `/root/vionsky/src/lib/auth-cookie.ts:47-86,99-112` `/root/vionsky/rust-workers/src/auth.rs:226-329` `/root/vionsky/API_CONTRACT.md:34-36`
- **cookie / chunking:** Large sessions are chunked as `sb-<ref>-auth-token.0`, `.1`, ... (MAX_CHUNK_SIZE=3180 per @supabase/ssr chunker). Reader prefers exact cookie; otherwise joins chunks sorted by numeric index, requires starting at .0 and strictly contiguous (missing/duplicate chunk => null). Rust port identical. `/root/vionsky/src/lib/auth-cookie.ts:16-45` `/root/vionsky/rust-workers/src/auth.rs:183-224,687-724` `/root/vionsky/API_CONTRACT.md:34-36`
- **cookie / http_only:** Cookie is HttpOnly, set by Supabase PKCE/OAuth flow; JS must not read it. Required UI auth method is cookie with credentials: same-origin. `/root/vionsky/API_CONTRACT.md:26-36` `/root/vionsky/src/utils/supabase/proxy.ts:4-9`
- **fetchWithAuth:** fetchWithAuth(input, init) simply calls fetch(input, {...init, credentials: 'same-origin'}) — no token read, no Authorization header attached by the browser. Do not attach Authorization from browser; only /api/v1/* scripts/agents may use Bearer (API key or JWT), server falls back to cookie when header absent. `/root/vionsky/src/lib/api.ts:1-10` `/root/vionsky/API_CONTRACT.md:30-33,266-269`
- **bearer_accepted:** extractTokenForRequest() prioritizes case-sensitive `Authorization: Bearer <token>`, then falls back to Supabase cookie; used by verifyUser / verifyAdmin / verifyModerator. `/root/vionsky/src/lib/auth.ts:58-74,99-136,166-200`
- **bearer_accepted:** Rust worker mirrors this: extract_bearer_token() then same cookie decode; verify_user / verify_admin / verify_moderator. `/root/vionsky/rust-workers/src/auth.rs:113-131,331-346,505-608`
- **bearer_accepted:** REST v1 authV1() accepts `Authorization: Bearer <API key>` (vsk_live_...) or `Authorization: Bearer <Supabase JWT>`; API key checked first, then verifyUser (Bearer JWT or HttpOnly cookie). No Bearer is not an immediate 401. `/root/vionsky/src/lib/v1-auth.ts:16-31,133-141,241-268` `/root/vionsky/API_CONTRACT.md:244-249`
- **bearer_accepted:** Role model user -> moderator -> admin. Admin = verified email equals ADMIN_EMAIL secret; moderator = users.role in D1; never trust client-supplied email/role. isAdminEmail() is single source of truth. `/root/vionsky/API_CONTRACT.md:41-43` `/root/vionsky/src/lib/auth.ts:76-93,166-200` `/root/vionsky/CORE.md:163`
- **api_key / format:** `vsk_live_` prefix + 40 random base62 chars (~48 chars, ~238-bit entropy); first 16 chars stored as key_prefix for display. Distinguishes API key from Supabase JWT via isApiKeyToken() prefix check. `/root/vionsky/src/lib/api-keys.ts:34-38,82-134`
- **api_key / scopes:** Valid scopes (CSV on key): `post:write`, `product:submit`, `comment:write`, `upvote:write`. No `admin:*` scope exists; key can never replace an admin session. requireScope(): JWT always passes, key missing scope -> 403 MISSING_SCOPE. `/root/vionsky/src/lib/api-keys.ts:28-54` `/root/vionsky/src/lib/v1-auth.ts:30,271-299` `/root/vionsky/content/llms/02-api.md:14-19`
- **api_key / storage:** Only SHA-256 hex hash stored in D1 api_keys (migration 020); plaintext returned exactly once from POST /api/v1/keys. verify checks existence, not revoked, not expired; timing-safe compare by hash lookup. `/root/vionsky/src/lib/api-keys.ts:6-25,116-129,169-204` `/root/vionsky/API_CONTRACT.md:249,258`
- **api_key / key_routes:** GET /api/v1/keys (JWT) lists keys + available_scopes; POST creates (name, scopes, expires_in_days?) and returns secret once; DELETE?key_id= revokes. v1 writes: /api/v1/products, /posts, /comments, /upvotes; GET /api/v1/openapi.json public. `/root/vionsky/API_CONTRACT.md:251-260` `/root/vionsky/src/app/api/v1/keys/route.ts:122-127`

### 4.2 Rate limit (app + edge)

- **storage:** App-level rate limits live in Cloudflare KV binding RATE_LIMIT_KV (id 27b4e12d52504799a4ae07b2823ac00c), NOT D1 (api_rate_limits table dropped in migration 028). Key format `rl:<scope>:<subject>:<bucket>`; TTL = window length; daily bucket = UTC YYYY-MM-DD, window bucket = fixed ISO window. GET -> compare limit -> PUT (non-atomic); missing KV/error => fail-closed 429. `/root/vionsky/src/lib/rate-limit.ts:5-18,231-249,259-317,319-338,466-527,543-611` `/root/vionsky/wrangler.jsonc:151-154` `/root/vionsky/rust-workers/wrangler.toml:105-117`
- **scopes_and_limits:** game_submit = 2 product submissions / user / day `/root/vionsky/src/lib/rate-limit.ts:22,63-65`
- **scopes_and_limits:** submit_analyze = 10 URL analyses / user / hour `/root/vionsky/src/lib/rate-limit.ts:23,65-69` `/root/vionsky/src/lib/submit-analyze.ts:6` `/root/vionsky/src/app/api/submit/analyze/route.ts:51`
- **scopes_and_limits:** bridge_send = 10 messages / room_id / minute `/root/vionsky/src/lib/rate-limit.ts:24,70-71` `/root/vionsky/src/lib/bridge.ts:6-7` `/root/vionsky/src/app/api/bridge/send/route.ts:170-175`
- **scopes_and_limits:** bridge_poll = 60 polls / agent (subject room_id:agent_id) / minute; counts HTTP requests, not long-poll inner loops `/root/vionsky/src/lib/rate-limit.ts:72-73` `/root/vionsky/src/app/api/bridge/poll/route.ts:43-51`
- **scopes_and_limits:** room_create = 5 rooms / user / hour `/root/vionsky/src/lib/rate-limit.ts:25,74-75` `/root/vionsky/docs/runbooks/a2a-chat-room.md:91`
- **scopes_and_limits:** report = 5 reports / user / day (MAX_REPORTS_PER_DAY=5) `/root/vionsky/src/lib/rate-limit.ts:26` `/root/vionsky/src/app/api/reports/route.ts:15,71-77`
- **scopes_and_limits:** upvote = 100 product upvotes / user / day (MAX_UPVOTES_PER_DAY=100) `/root/vionsky/src/lib/rate-limit.ts:27` `/root/vionsky/src/app/api/upvotes/route.ts:16,109`
- **scopes_and_limits:** post_upvote = 100 post upvotes / user / day (separate scope from product upvote) `/root/vionsky/src/lib/rate-limit.ts:28,80-87` `/root/vionsky/src/app/api/posts/[id]/upvote/route.ts:15,72` `/root/vionsky/src/app/api/v1/upvotes/route.ts:15,88-93`
- **scopes_and_limits:** follow = 20 follows / user / day (MAX_FOLLOWS_PER_DAY=20) `/root/vionsky/src/lib/rate-limit.ts:29` `/root/vionsky/src/app/api/users/[id]/follow/route.ts:20,76`
- **scopes_and_limits:** search = 30 searches / IP / minute (SEARCH_RATE_MAX=30, window 60s, subject ip:<CF-Connecting-IP>) `/root/vionsky/src/lib/rate-limit.ts:30,90-96` `/root/vionsky/src/app/api/search/route.ts:9-10,57-63`
- **scopes_and_limits:** post_create = 3 posts / user / 10 minutes (POST_RATE_MAX=3, window 600s), for both /api/posts and /api/v1/posts `/root/vionsky/src/lib/rate-limit.ts:31,97-98` `/root/vionsky/src/app/api/posts/route.ts:26-27,206-207` `/root/vionsky/src/app/api/v1/posts/route.ts:21-22,122-127`
- **scopes_and_limits:** post_comment = 10 comments / user / minute (COMMENT_RATE_MAX=10, window 60s) `/root/vionsky/src/lib/rate-limit.ts:32,99-100` `/root/vionsky/src/app/api/v1/comments/route.ts:12-13,113-118`
- **scopes_and_limits:** review = 3 reviews of one product / user / 10 minutes `/root/vionsky/src/lib/rate-limit.ts:33,101-102`
- **scopes_and_limits:** community_image_upload = 5 images / user / day (MAX_UPLOADS_PER_DAY=5) for POST /api/upload `/root/vionsky/src/lib/rate-limit.ts:34,132-139` `/root/vionsky/src/app/api/upload/route.ts:15,90`
- **scopes_and_limits:** apikey_create = 20 keys / user / 10 minutes (KEY_CREATE_MAX=20, window 600s, subject `apikey:<id>`) `/root/vionsky/src/lib/rate-limit.ts:103-108` `/root/vionsky/src/app/api/v1/keys/route.ts:25-26,122-127` `/root/vionsky/API_CONTRACT.md:70`
- **scopes_and_limits:** apikey_action = 50 actions / key / hour (KEY_HOURLY_ACTION_LIMIT=50, window 3600s); exceeding auto-revokes the key (CODE KEY_AUTO_REVOKED 429) and notifies owner. Separate upvote guardrail 50/hour/key. `/root/vionsky/src/lib/rate-limit.ts:109-115` `/root/vionsky/src/lib/api-key-guardrail.ts:18-29` `/root/vionsky/src/lib/v1-auth.ts:186-222`
- **scopes_and_limits:** visky_chat = 10 chats / IP / minute (subject ip:<CF-Connecting-IP>) for POST /api/visky/chat `/root/vionsky/src/lib/rate-limit.ts:116-123` `/root/vionsky/docs/VISKY_GATEWAY.md:274-276,381` `/root/vionsky/API_CONTRACT.md:69,229`
- **scopes_and_limits:** visky_global_daily = 100 chats / day UTC system-wide (subject fixed `global`), shared to protect free-tier provider keys `/root/vionsky/src/lib/rate-limit.ts:124-131` `/root/vionsky/docs/VISKY_GATEWAY.md:10,382` `/root/vionsky/API_CONTRACT.md:69,229`
- **scopes_and_limits:** games_lookup = 60 lookups / IP / minute (documented; enforced in Rust layer, not listed in src/lib/rate-limit.ts) `/root/vionsky/API_CONTRACT.md:60` `/root/vionsky/CORE.md:131`
- **bypass:** Admin/moderator bypass requires ALL 3: kill-switch env RATE_LIMIT_ROLE_BYPASS === 'true' (set in both wrangler configs), scope in allowlist {game_submit, upvote, post_upvote, post_create, post_comment, review}, role exactly 'admin' or 'moderator'. Never applies to search (IP), apikey_create/apikey_action (per-key), report/follow, bridge_send/bridge_poll/room_create. Bypass only after KV already reported quota exhausted; KV errors stay fail-closed. `/root/vionsky/src/lib/rate-limit.ts:42-54,145-220` `/root/vionsky/wrangler.jsonc:44-47` `/root/vionsky/rust-workers/wrangler.toml:64-66`
- **cloudflare_zone_rate_limit / rule:** Zone ruleset `default`, phase `http_ratelimit`, version 2, rule id 2a5b5eb3d6114afab33d46d90ccee5a1, description `Protect API`, enabled=true, action=block; expression: (http.request.uri.path contains "/api/" and not starts_with(http.request.uri.path, "/api/auth/session") and not starts_with(http.request.uri.path, "/api/notifications")); ratelimit: requests_per_period=30, period=10 seconds, mitigation_timeout=10 seconds, characteristics=[ip.src, cf.colo.id]; last_updated 2026-11-24T20:09:09Z. `https://api.cloudflare.com/client/v4/zones/dec5d312fdb3c01ac115d26c86fda187/rulesets/b89e547f87984bd19a32d2d41bf2eb55`
- **cloudflare_zone_rate_limit / notes:** This is an edge/zone rule, separate from the app-level KV limits. The legacy `/rate_limits` Cloudflare endpoint is deprecated and returns a message directing to the Rulesets API. `https://api.cloudflare.com/client/v4/zones/dec5d312fdb3c01ac115d26c86fda187/rate_limits`

## 5. WebSocket / A2A

- **public_websocket:** NONE for the frontend: no browser-facing WebSocket endpoint and no WS route in src/app/api. Browser <-> Worker is HTTP REST plus SSE (`text/event-stream`) at POST /api/visky/chat; Content-Type JSON except /api/upload (multipart) and /api/visky/chat (SSE). `/root/vionsky/API_CONTRACT.md:19-20,227` `/root/vionsky/docs/VISKY_GATEWAY.md:23-26,243-272`
- **internal_websocket:** The only WebSocket is server-side Worker/Rust -> ZeroClaw Gateway over Cloudflare Tunnel: wss://visky.vionsky.com/ws/chat?session_id=<id>&agent=<alias>&token=<token>, subprotocol `zeroclaw.v1`, token in query string. Gateway binds 127.0.0.1:42617 and is not public. Workers open outbound WS via fetch(url,{headers:{Upgrade:'websocket'}}) with `https://` scheme (wss:// is rejected by fetch); Rust uses Sec-WebSocket-Protocol zeroclaw.v1. Proxied to browser as SSE, so the WS is not a client contract. `/root/vionsky/src/lib/visky/zeroclaw.ts:10-37,66-89,127-144,176-181` `/root/vionsky/docs/VISKY_GATEWAY.md:28,41-45,309-338,295-307,369` `/root/vionsky/rust-workers/src/lib.rs:7118-7135`
- **a2a_protocol_docs:** No standard A2A protocol spec: zero occurrences of agent card / .well-known/agent.json / Agent2Agent / 'A2A protocol' anywhere in the repo. 'A2A' here means a bespoke chat-room bridge over HTTP long-poll, not the Agent2Agent protocol. Docs: docs/runbooks/a2a-chat-room.md (current), docs/archive/agent-bridge.md (old global BRIDGE_SECRET cutover, do not use), content/llms/02-api.md A2A section. `/root/vionsky/docs/runbooks/a2a-chat-room.md:1-12` `/root/vionsky/docs/archive/agent-bridge.md:1-7` `/root/vionsky/content/llms/02-api.md:23-33`
- **long_poll_alternative:** UI: GET /api/rooms/<roomId>/messages?after=<lastId>&wait=<0..25> (owner JWT; MAX_WAIT_SECONDS=25, internal 500ms DB poll, <=50 messages) returns when a message arrives or wait expires; `no-store` + `CDN-Cache-Control: no-store`; UI re-polls immediately. Runbook describes 3s UI poll and backoff 0.5s->1s->2s->4s. `/root/vionsky/src/app/api/rooms/[roomId]/messages/route.ts:6-9,26-70` `/root/vionsky/API_CONTRACT.md:172,270-273` `/root/vionsky/docs/runbooks/a2a-chat-room.md:11,21,80,93`
- **long_poll_alternative:** Agents: GET /api/bridge/poll?room_id=<id>&agent_id=<name>&wait=<0..25> with header x-room-secret, MAX_WAIT_SECONDS=25; rate limit 60 requests/min per room:agent; presence written to KV TTL 60s. `/root/vionsky/src/app/api/bridge/poll/route.ts:20-22,33-61,80-95` `/root/vionsky/content/llms/02-api.md:27-33`
- **long_poll_alternative:** Other A2A endpoints: POST /api/bridge/send (10 msg/room/min), GET /api/bridge/history (50 recent), plus room management /api/rooms/* and /api/rooms/<id>/agents|presence|sticky|rotate. Auth for agents = per-room `x-room-secret` (plaintext shown once, stored as SHA-256, timing-safe compare); no global BRIDGE_SECRET. Max 50 active agents/room; free tier 2 rooms. `/root/vionsky/docs/runbooks/a2a-chat-room.md:13-26,48-58,75-93` `/root/vionsky/API_CONTRACT.md:162-182`
- **long_poll_alternative:** Rust forwards /api/bridge/* and /api/rooms/* to Next.js/OpenNext (these routes are not served directly by Rust handlers). `/root/vionsky/rust-workers/src/lib.rs:6431` `/root/vionsky/CORE.md:134-145`

### 5.1 SSE — `POST /api/visky/chat` (thay cho WebSocket)

Request: `{ message: string, model?: string, sessionId?: string, history?: Array<{role:string,content:string}> }`
(history tối đa 40 item, mỗi content tối đa 600 ký tự). Response `Content-Type: text/event-stream`.

```text
data: {"type":"start"}
data: {"type":"chunk","content":"Xin chào"}
data: {"type":"tool_use","name":"search"}
data: {"type":"done","sessionId":"..."}
data: {"type":"error","code":"VISKY_UPSTREAM_ERROR","message":"..."}
```

### 5.2 Long-poll A2A (không phải Agent2Agent protocol chuẩn)

```bash
# UI (owner, cookie/Bearer):
fetch('/api/rooms/<roomId>/messages?after=<lastId>&wait=25', { credentials:'same-origin', cache:'no-store' })
# Agent (room secret):
fetch('/api/bridge/poll?room_id=<id>&agent_id=<name>&wait=25', { headers:{'x-room-secret': ROOM_SECRET} })
fetch('/api/bridge/send', { method:'POST', headers:{'x-room-secret': ROOM_SECRET,'Content-Type':'application/json'},
  body: JSON.stringify({room_id, sender_id:'min', receiver_id:'user', content:'hello', parent_id?, end?}) })
```

- `wait` = 0..25 giây (MAX_WAIT_SECONDS=25); khi hết hạn trả `{success:true,messages:[]}`, UI poll lại ngay.
- Mọi response rooms/messages: `Cache-Control: no-store` + `CDN-Cache-Control: no-store`.
- `room_secret` chỉ trả 1 lần lúc tạo phòng / rotate; agent gửi qua header `x-room-secret`, KHÔNG dùng `BRIDGE_SECRET` global.
- Loop guard: A→B rồi B→A trong 15s → `429 RATE_LIMITED reason=loop_guard`. Depth > 10 → `CONVERSATION_TOO_DEEP`.

## 6. Base URL & routing

- **production:** Production base URL https://vionsky.com; all APIs are same-origin relative `/api/...` with NO /vi or /en locale prefix. Do not hardcode internal URLs. `/root/vionsky/API_CONTRACT.md:12,264-267` `/root/vionsky/wrangler.jsonc:35`
- **custom_domain:** Cloudflare custom domain `vionsky.com` -> Worker `vionsky-web` (service vionsky-web, environment production, id 90209747a16c3ce79479e63ac2e729e14d1d5a49). Zone id dec5d312fdb3c01ac115d26c86fda187 (Free plan). Account id 37a208d8b6b854d5cf0204d3d3e3d912. workers_dev:false, Workers Cache enabled. `https://api.cloudflare.com/client/v4/accounts/37a208d8b6b854d5cf0204d3d3e3d912/workers/domains` `/root/vionsky/HANDOVER.md:132-139,148-153` `/root/vionsky/wrangler.jsonc:8-13,41`
- **routing:** All /api/* hit Rust Worker `vionsky-rust-pilot` via zone routes (zone route wins over the custom domain for matching paths); some specific patterns (upvotes/batch, games/lookup, search, upvotes) precede catchall `vionsky.com/api/*`. Rust serves migrated handlers; unmigrated ones forward via Service Binding `VIONSKY_WEB` -> vionsky-web (OpenNext), whose responses carry `x-opennext: 1`. Next never calls Rust. `/root/vionsky/rust-workers/wrangler.toml:14-53,87-93` `https://api.cloudflare.com/client/v4/zones/dec5d312fdb3c01ac115d26c86fda187/workers/routes` `/root/vionsky/API_CONTRACT.md:13-17,275-276`
- **other_hosts:** visky.vionsky.com is a Cloudflare Tunnel hostname to ZeroClaw Gateway 127.0.0.1:42617 (not a Worker custom domain); VISKY_ZEROCLAW_URL is a public var, VISKY_ZEROCLAW_TOKEN is a Worker secret. img.vionsky.com is only a recommended custom domain for the vionsky-images R2 bucket (runbook), not confirmed as deployed. `/root/vionsky/docs/VISKY_GATEWAY.md:284-307,369` `/root/vionsky/wrangler.jsonc:48-55,86-92`

### 6.1 Edge rate-limit (Cloudflare zone rule)

- **rule:** Zone ruleset `default`, phase `http_ratelimit`, version 2, rule id 2a5b5eb3d6114afab33d46d90ccee5a1, description `Protect API`, enabled=true, action=block; expression: (http.request.uri.path contains "/api/" and not starts_with(http.request.uri.path, "/api/auth/session") and not starts_with(http.request.uri.path, "/api/notifications")); ratelimit: requests_per_period=30, period=10 seconds, mitigation_timeout=10 seconds, characteristics=[ip.src, cf.colo.id]; last_updated 2026-11-24T20:09:09Z. `https://api.cloudflare.com/client/v4/zones/dec5d312fdb3c01ac115d26c86fda187/rulesets/b89e547f87984bd19a32d2d41bf2eb55`
- **notes:** This is an edge/zone rule, separate from the app-level KV limits. The legacy `/rate_limits` Cloudflare endpoint is deprecated and returns a message directing to the Rulesets API. `https://api.cloudflare.com/client/v4/zones/dec5d312fdb3c01ac115d26c86fda187/rate_limits`

## 7. Nguồn & kiểm chứng

- Đã quét **60** file `src/app/api/**/route.ts` → **82** method+path Next.
- Đã quét `rust-workers/src/lib.rs` + `auth.rs` → **41** entry Rust (handler + forward).
- Cloudflare Rulesets API (zone `dec5d312fdb3c01ac115d26c86fda187`) để xác nhận rule rate-limit.
- Auth/rate-limit/SSE/A2A đối chiếu: `src/lib/auth.ts`, `auth-cookie.ts`, `api.ts`, `rate-limit.ts`, `v1-auth.ts`,
  `docs/runbooks/a2a-chat-room.md`, `docs/VISKY_GATEWAY.md`, `src/app/api/visky/chat/route.ts`.

> Khi thêm/sửa API: cập nhật file này **trong cùng commit**, rồi báo Culi pull. Không để 2 bản contract tồn tại song song.

