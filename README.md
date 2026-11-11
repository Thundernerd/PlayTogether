# PlayTogether

Compare Steam wishlists with friends, mark games you want to play together, and vote on what to play next.

## Prerequisites

- Rust (stable)
- Node.js 22.12+
- Steam profiles with **public** wishlists and **public game details** (required for sync)
- A [Steam Web API key](https://steamcommunity.com/dev/apikey) (required for owned-games library sync)

## Run locally

### Backend (port 8080)

```bash
cd backend
cargo run
```

Create a `backend/.env` file (optional) to set variables locally — it is loaded automatically on startup:

```bash
STEAM_API_KEY=your-steam-web-api-key
```

Environment variables (optional):

| Variable | Default |
|---|---|
| `DATABASE_URL` | `sqlite:playtogether.db?mode=rwc` |
| `BIND_ADDR` | `0.0.0.0:8080` |
| `GAME_CACHE_TTL_DAYS` | `7` |
| `PRICE_CACHE_TTL_HOURS` | `24` |
| `STEAM_CHUNK_SIZE` | `50` |
| `STEAM_CHUNK_DELAY_MS` | `200` |
| `STEAM_API_KEY` | *(unset — wishlist sync works; library sync is skipped)* |

If you have an existing `playwishlist.db` from a previous install, rename it to `playtogether.db` or set `DATABASE_URL=sqlite:playwishlist.db?mode=rwc`.

### Frontend (port 3000)

```bash
cd frontend
npm install
npm run dev
```

Set `BACKEND_URL=http://localhost:8080` if the API runs elsewhere.

Type-check the frontend (including `.vue` files) with `npm run typecheck`; `npm run build` does not type-check.

## How it works

1. **Create or join a group** using an invite link (no accounts — a browser token identifies you).
2. **Add your Steam ID64** — the backend fetches your public wishlist and owned games from Steam.
3. **Game metadata** is fetched in chunks and stored in SQLite (`steam_games`). Cached data is reused across all members and groups for 7 days (prices for 24 hours). Games with no cached row are fetched before a list loads; rows past either TTL are served from the cache and refreshed in the background.
4. **Check games** you want to play together on your wishlist or library cards.
5. **Vote and rate** games in the shared “Play together” list.

## API overview

- `POST /api/groups` — create group
- `POST /api/groups/by-invite/:code/join` — join group
- `PATCH /api/members/me` — set Steam ID (triggers sync)
- `DELETE /api/members/me` — leave group (removes your picks, votes, and synced games)
- `GET /api/members/me/sync-status` — background sync progress
- `GET /api/groups/:id/wishlists` — all member wishlists
- `GET /api/groups/:id/libraries` — all member owned-game libraries
- `GET /api/groups/:id/shared-games` — aggregated picks with votes

All authenticated routes require the `X-Member-Token` header returned on create/join.
