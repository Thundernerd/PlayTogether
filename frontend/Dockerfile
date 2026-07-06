# syntax=docker/dockerfile:1

FROM node:20-alpine AS builder

WORKDIR /app

ARG NUXT_PUBLIC_API_BASE=http://localhost:8080
ENV NUXT_PUBLIC_API_BASE=${NUXT_PUBLIC_API_BASE}

COPY package.json package-lock.json ./
RUN npm ci

COPY . .
RUN npm run build

FROM node:20-alpine AS runtime

WORKDIR /app

ENV NODE_ENV=production \
    NITRO_HOST=0.0.0.0 \
    NITRO_PORT=3000

ARG NUXT_PUBLIC_API_BASE=http://localhost:8080
ENV NUXT_PUBLIC_API_BASE=${NUXT_PUBLIC_API_BASE}

COPY --from=builder /app/.output ./.output

EXPOSE 3000

CMD ["node", ".output/server/index.mjs"]
