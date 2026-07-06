#!/usr/bin/env bash
set -euo pipefail

TAG="${1:-latest}"
REGISTRY="tnrd"

docker build -t "${REGISTRY}/playtogether-backend:${TAG}" ./backend
docker build \
  --build-arg NUXT_PUBLIC_API_BASE="${NUXT_PUBLIC_API_BASE:-http://localhost:8080}" \
  -t "${REGISTRY}/playtogether-frontend:${TAG}" \
  ./frontend

docker push "${REGISTRY}/playtogether-backend:${TAG}"
docker push "${REGISTRY}/playtogether-frontend:${TAG}"
