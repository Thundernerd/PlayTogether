# PlayTogether

Compare Steam wishlists with friends, mark games you want to play together, and vote on what to play next.

## Prerequisites

- Rust (stable)
- Node.js 20+
- Steam profiles with **public** wishlists (required for sync)

## Run locally

### Backend (port 8080)

```bash
cd backend
cargo run
```

Environment variables (optional):

| Variable | Default |
|---|---|
| `DATABASE_URL` | `sqlite:playtogether.db?mode=rwc` |
| `BIND_ADDR` | `0.0.0.0:8080` |
| `CORS_ORIGIN` | `http://localhost:3000` |
| `GAME_CACHE_TTL_DAYS` | `7` |
| `PRICE_CACHE_TTL_HOURS` | `24` |
| `STEAM_CHUNK_SIZE` | `50` |
| `STEAM_CHUNK_DELAY_MS` | `200` |

If you have an existing `playwishlist.db` from a previous install, rename it to `playtogether.db` or set `DATABASE_URL=sqlite:playwishlist.db?mode=rwc`.

### Frontend (port 3000)

```bash
cd frontend
npm install
npm run dev
```

Set `NUXT_PUBLIC_API_BASE=http://localhost:8080` if the API runs elsewhere.

## How it works

1. **Create or join a group** using an invite link (no accounts — a browser token identifies you).
2. **Add your Steam ID64** — the backend fetches your public wishlist from Steam.
3. **Game metadata** is fetched in chunks and stored in SQLite (`steam_games`). Cached data is reused across all members and groups for 7 days.
4. **Check games** you want to play together on your wishlist cards.
5. **Vote and rate** games in the shared “Play together” list.

## API overview

- `POST /api/groups` — create group
- `POST /api/groups/by-invite/:code/join` — join group
- `PATCH /api/members/me` — set Steam ID (triggers sync)
- `GET /api/members/me/sync-status` — background sync progress
- `GET /api/groups/:id/wishlists` — all member wishlists
- `GET /api/groups/:id/shared-games` — aggregated picks with votes

All authenticated routes require the `X-Member-Token` header returned on create/join.
