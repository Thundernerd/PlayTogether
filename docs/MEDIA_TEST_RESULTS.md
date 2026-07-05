# Media test results

Generated from the [media test plan](/.cursor/plans/media_test_plan_bea255a1.plan.md) using appids **570** (legacy control), **4680930**, and **2478130**.

**Run automated checks:**

```bash
# Terminal 1
cd backend && cargo run

# Terminal 2
./scripts/verify-media.sh http://localhost:8080
```

**Manual API inspection (Layer 2):**

```bash
API=http://localhost:8080
curl -sS "$API/api/games/4680930" | jq '{
  appid, name,
  header: .header_image_url,
  screenshot_count: (.screenshots | length),
  trailers: [.trailers[] | {name, thumb: .thumbnail_url, video: .video_url, webm: .video_webm_url}]
}'
```

Repeat for `2478130` and `570`.

---

## Layer 1 — Unit tests

**Command:** `cd backend && CARGO_TARGET_DIR=target cargo test steam::media_urls`

| Test | 570 | 4680930 | 2478130 |
|---|---|---|---|
| Header URL | PASS | PASS | — |
| Screenshot URL | PASS | PASS | — |
| Trailer thumb URL | PASS (legacy) | PASS | PASS (2 trailers) |
| Trailer video URL | PASS (movie480) | PASS (microtrailer) | PASS (microtrailer) |
| Allowlist | — | PASS | PASS |

Fixtures live in [`backend/src/steam/media_urls.rs`](../backend/src/steam/media_urls.rs) (`trailer_4680930`, `trailer_2478130_*` helpers).

---

## Layers 2–4 — API, CDN, proxy (automated)

**Last run:** `./scripts/verify-media.sh` — **49 passed, 0 failed**

| Test | 570 | 4680930 | 2478130 |
|---|---|---|---|
| API: detail JSON complete | PASS | PASS (7 screenshots, 1 trailer) | PASS (18 screenshots, 2 trailers) |
| CDN HEAD: header | PASS | PASS | PASS |
| CDN HEAD: screenshot[0] | PASS | PASS | PASS |
| CDN HEAD: trailer thumb | PASS (×5) | PASS | PASS (×2) |
| Proxy: video_url path | PASS | PASS | PASS |
| Proxy HEAD mp4 | PASS | PASS | PASS |
| Proxy Range 0–1023 | PASS (206, 1024 B) | PASS | PASS |
| Proxy HEAD webm | PASS | PASS | PASS |

---

## Layer 5 — Browser manual checklist

Confirm in the UI after a hard refresh (clears `game-detail-cache-v2`).

### Images

| Step | 4680930 | 2478130 |
|---|---|---|
| Wishlist card header (not shimmer) | ☐ | ☐ |
| Modal header slide loads | ☐ | ☐ |
| Screenshot thumbs in carousel strip | ☐ (7) | ☐ (18) |
| Trailer thumb(s) visible | ☐ (1) | ☐ (2) |
| Network tab: image requests 200 | ☐ | ☐ |

### Video

| Step | 4680930 | 2478130 |
|---|---|---|
| Trailer slide plays (Gameplay) | ☐ | ☐ (Cinematic) |
| Second trailer plays after switching slide | n/a | ☐ (Gameplay) |
| Network: video → `localhost:8080/api/games/.../trailers/.../video` | ☐ | ☐ |
| Response 200/206, `Content-Type: video/*` | ☐ | ☐ |
| Checkbox/vote does not open modal | ☐ | ☐ |

**If video fails in browser but script passes:** check DevTools → Network for the proxy URL, compare with `curl -I` on the same URL, and inspect `<video>` `MediaError` code (2=network, 4=unsupported codec).

---

## Layer 6 — End-to-end matrix

| Test | 570 | 4680930 | 2478130 |
|---|---|---|---|
| Unit: header URL | PASS | PASS | — |
| Unit: screenshot URL | PASS | PASS | — |
| Unit: trailer thumb URL | PASS | PASS | PASS |
| Unit: trailer video URL | PASS | PASS | PASS |
| API: detail JSON | PASS | PASS | PASS |
| CDN HEAD: header | PASS | PASS | PASS |
| CDN HEAD: screenshot[0] | PASS | PASS | PASS |
| CDN HEAD: trailer thumb | PASS | PASS | PASS |
| Proxy HEAD mp4 | PASS | PASS | PASS |
| Proxy Range 0–1023 | PASS | PASS | PASS |
| Browser: card thumbnail | Manual | Manual | Manual |
| Browser: modal video plays | Manual | Manual | Manual |

---

## Root cause (fixed)

Newer Steam titles store trailers as **microtrailer** files on `video.akamai.steamstatic.com/store_trailers/`, not legacy `movie480.mp4` on Cloudflare. Resolver and proxy allowlist were updated in `media_urls.rs` to:

- Build trailer thumbs from `trailer_url_format` + full `screenshot_medium` path
- Prefer `microtrailer` mp4/webm for playback
- Allow `video.akamai.steamstatic.com/store_trailers/` in the proxy

Legacy app **570** still uses `movie480.mp4` and remains covered by regression tests.
