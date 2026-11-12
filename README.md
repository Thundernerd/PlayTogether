# PlayTogether

Compare Steam wishlists with friends, mark games you want to play together, and vote on what to play next.

## Prerequisites

- Rust, as pinned in `backend/rust-toolchain.toml` (see [Backend](#backend-port-8080))
- Node.js 22.18+ (or 24.11+)
- npm as pinned in `frontend/package.json`'s `packageManager` field (see [Frontend](#frontend-port-3000))
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

`backend/rust-toolchain.toml` is the source of truth for the Rust version. rustup installs and uses it automatically in `backend/`, and so does the "Backend checks" workflow. To bump the version, edit:

- `channel` in `backend/rust-toolchain.toml`
- the `RUST_VERSION` build arg default in `backend/Dockerfile` (the Docker build fails if the two differ)

Then run `cargo fmt` and `cargo clippy --all-targets` in `backend/` and fix anything the new version reports.

The "Backend checks" workflow fails if the backend isn't `rustfmt`-clean, has clippy warnings, or has a failing test. Before pushing, run these in `backend/`:

- `cargo fmt` (`cargo fmt --check` reports drift without changing files)
- `cargo clippy --all-targets -- -D warnings`
- `cargo test`

### Frontend (port 3000)

```bash
cd frontend
npm install
npm run dev
```

Set `BACKEND_URL=http://localhost:8080` if the API runs elsewhere.

`frontend/package-lock.json` is written by the npm pinned in `frontend/package.json`'s `packageManager` field, and CI and the Docker build install that same npm before `npm ci`. Other npm versions (such as npm 10, bundled with Node 22) resolve the lockfile differently and break `npm ci`, so install the pinned npm before changing dependencies. Run this from `frontend/`:

```bash
npm install -g "$(node -p "require('./package.json').packageManager")"
```

The pinned npm runs a dependency's install scripts only when `allowScripts` in `frontend/package.json` allows that package. The packages it denies don't need their scripts, because each one already ships a prebuilt binary: esbuild and @parcel/watcher through their optional platform packages, and fsevents (macOS only) inside the package. If `npm ci` warns that a new dependency's install scripts are `not yet covered by allowScripts`, decide whether it needs them and record the answer with `npm deny-scripts <pkg>` or `npm approve-scripts <pkg>`.

Type-check the frontend (including `.vue` files) with `npm run typecheck`; `npm run build` does not type-check.

Run the frontend unit tests (Vitest, under `frontend/test/`) with `npm run test`.

`frontend/.nvmrc` is the source of truth for the frontend's Node major. `nvm use` reads it, and so does the "Frontend checks" workflow. To change the major, edit:

- `frontend/.nvmrc`
- the `NODE_VERSION` build arg default in `frontend/Dockerfile`
- the `node:<major>-alpine` image tag in `docker-compose.dev.yml` (compose can't read `.nvmrc`)

The minimum supported version is `engines.node` in `frontend/package.json`; if it changes, update the prerequisite above too.

## How it works

1. **Create or join a group** using an invite link (no accounts — a browser token identifies you).
2. **Add your Steam ID64** — the backend fetches your public wishlist and owned games from Steam.
3. **Game metadata** is fetched in chunks and stored in SQLite (`steam_games`). Cached data is reused across all members and groups for 7 days (prices for 24 hours). Games with no cached row are fetched before a list loads, once even when list loads overlap; rows past either TTL are served from the cache and refreshed in the background. A game Steam fails for is not fetched again for 15 minutes, and a game with no cached row is left out of the lists until then.
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
