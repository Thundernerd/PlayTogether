# syntax=docker/dockerfile:1

FROM node:20-alpine AS builder

WORKDIR /app

# Proxy /api to the backend service on the compose network.
ENV NUXT_API_PROXY=http://backend:8080

COPY package.json package-lock.json ./
RUN npm ci

COPY . .
RUN npm run build

FROM node:20-alpine AS runtime

WORKDIR /app

ENV NODE_ENV=production \
    NITRO_HOST=0.0.0.0 \
    NITRO_PORT=3000

COPY --from=builder /app/.output ./.output

EXPOSE 3000

CMD ["node", ".output/server/index.mjs"]
