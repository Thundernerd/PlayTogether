#!/usr/bin/env bash
# Verify Steam media URLs for PlayTogether game detail API.
# Usage: ./scripts/verify-media.sh [API_BASE]
# Default API_BASE: http://localhost:8080

set -euo pipefail

API_BASE="${1:-http://localhost:8080}"
APPIDS=(570 4680930 2478130)

pass=0
fail=0
results=()

record() {
  local status="$1"
  local label="$2"
  if [[ "$status" == "PASS" ]]; then
    pass=$((pass + 1))
    results+=("PASS  $label")
  else
    fail=$((fail + 1))
    results+=("FAIL  $label  ($status)")
  fi
}

head_status() {
  local url="$1"
  curl -sS -o /dev/null -w "%{http_code}" -I "$url" 2>/dev/null || echo "000"
}

head_content_type() {
  local url="$1"
  curl -sS -I "$url" 2>/dev/null | awk -F': ' 'tolower($1)=="content-type"{print $2; exit}' | tr -d '\r'
}

proxy_range_status() {
  local url="$1"
  curl -sS -o /dev/null -w "%{http_code}" -H "Range: bytes=0-1023" "$url" 2>/dev/null || echo "000"
}

proxy_body_size() {
  local url="$1"
  curl -sS -o /dev/null -w "%{size_download}" -H "Range: bytes=0-1023" "$url" 2>/dev/null || echo "0"
}

echo "PlayTogether media verification"
echo "API: $API_BASE"
echo

for appid in "${APPIDS[@]}"; do
  echo "=== appid $appid ==="

  detail_json="$(curl -sS "$API_BASE/api/games/$appid" 2>/dev/null || true)"
  if [[ -z "$detail_json" ]]; then
    record "no API response" "appid $appid: GET /api/games/$appid"
    echo
    continue
  fi

  if ! echo "$detail_json" | jq -e .appid >/dev/null 2>&1; then
    record "$(echo "$detail_json" | jq -r '.error // "invalid JSON"')" "appid $appid: GET /api/games/$appid"
    echo
    continue
  fi

  record "PASS" "appid $appid: GET /api/games/$appid"

  header_url="$(echo "$detail_json" | jq -r '.header_image_url // empty')"
  if [[ -n "$header_url" && "$header_url" == https://shared.fastly.steamstatic.com/store_item_assets/* ]]; then
    code="$(head_status "$header_url")"
    if [[ "$code" == "200" ]]; then
      record "PASS" "appid $appid: CDN HEAD header ($code)"
    else
      record "HTTP $code" "appid $appid: CDN HEAD header"
    fi
  else
    record "bad header URL" "appid $appid: header_image_url shape"
  fi

  screenshot_count="$(echo "$detail_json" | jq '.screenshots | length')"
  if [[ "$screenshot_count" -gt 0 ]]; then
    shot_url="$(echo "$detail_json" | jq -r '.screenshots[0]')"
    code="$(head_status "$shot_url")"
    if [[ "$code" == "200" ]]; then
      record "PASS" "appid $appid: CDN HEAD screenshot[0] ($code)"
    else
      record "HTTP $code" "appid $appid: CDN HEAD screenshot[0]"
    fi
  else
    record "no screenshots" "appid $appid: CDN HEAD screenshot[0]"
  fi

  trailer_count="$(echo "$detail_json" | jq '.trailers | length')"
  echo "  trailers: $trailer_count, screenshots: $screenshot_count"

  for ((i = 0; i < trailer_count; i++)); do
    thumb_url="$(echo "$detail_json" | jq -r ".trailers[$i].thumbnail_url // empty")"
    video_path="$(echo "$detail_json" | jq -r ".trailers[$i].video_url // empty")"
    webm_path="$(echo "$detail_json" | jq -r ".trailers[$i].video_webm_url // empty")"
    trailer_name="$(echo "$detail_json" | jq -r ".trailers[$i].name // \"trailer $i\"")"

    if [[ -n "$thumb_url" ]]; then
      code="$(head_status "$thumb_url")"
      if [[ "$code" == "200" ]]; then
        record "PASS" "appid $appid trailer[$i] ($trailer_name): CDN HEAD thumb ($code)"
      else
        record "HTTP $code" "appid $appid trailer[$i] ($trailer_name): CDN HEAD thumb"
      fi
    else
      record "missing thumb" "appid $appid trailer[$i] ($trailer_name): CDN HEAD thumb"
    fi

    if [[ "$video_path" == /api/games/$appid/trailers/$i/video ]]; then
      record "PASS" "appid $appid trailer[$i]: proxy video_url path"
    else
      record "path=$video_path" "appid $appid trailer[$i]: proxy video_url path"
    fi

    if [[ -n "$video_path" ]]; then
      proxy_url="$API_BASE$video_path"
      head_code="$(head_status "$proxy_url")"
      ct="$(head_content_type "$proxy_url")"
      if [[ "$head_code" == "200" || "$head_code" == "206" ]]; then
        record "PASS" "appid $appid trailer[$i]: proxy HEAD mp4 ($head_code, $ct)"
      else
        record "HTTP $head_code" "appid $appid trailer[$i]: proxy HEAD mp4"
      fi

      range_code="$(proxy_range_status "$proxy_url")"
      body_size="$(proxy_body_size "$proxy_url")"
      if [[ "$range_code" == "206" && "$body_size" -gt 0 ]]; then
        record "PASS" "appid $appid trailer[$i]: proxy Range 0-1023 ($range_code, ${body_size}B)"
      else
        record "HTTP $range_code size=$body_size" "appid $appid trailer[$i]: proxy Range 0-1023"
      fi
    fi

    if [[ -n "$webm_path" ]]; then
      webm_url="$API_BASE$webm_path"
      webm_code="$(head_status "$webm_url")"
      webm_ct="$(head_content_type "$webm_url")"
      if [[ "$webm_code" == "200" || "$webm_code" == "206" ]]; then
        record "PASS" "appid $appid trailer[$i]: proxy HEAD webm ($webm_code, $webm_ct)"
      else
        record "HTTP $webm_code" "appid $appid trailer[$i]: proxy HEAD webm"
      fi
    fi
  done

  echo
done

echo "Summary ($pass passed, $fail failed)"
printf '%s\n' "${results[@]}"

if [[ "$fail" -gt 0 ]]; then
  exit 1
fi
