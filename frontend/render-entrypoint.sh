#!/bin/sh
set -eu

: "${BACKEND_UPSTREAM_URL:?BACKEND_UPSTREAM_URL must be set}"
: "${PORT:=10000}"
: "${ASSET_VERSION:=$(date +%s)}"

DNS_RESOLVER="${DNS_RESOLVER:-$(awk '/^nameserver[[:space:]]+/ { print $2; exit }' /etc/resolv.conf)}"

if [ -z "$DNS_RESOLVER" ]; then
    echo "Unable to determine a DNS resolver for nginx upstream lookups" >&2
    exit 1
fi

BACKEND_UPSTREAM_URL="${BACKEND_UPSTREAM_URL%/}"
ANALYTICS_UPSTREAM_URL="${ANALYTICS_UPSTREAM_URL:-$BACKEND_UPSTREAM_URL}"
ANALYTICS_UPSTREAM_URL="${ANALYTICS_UPSTREAM_URL%/}"
GRAPH_UPSTREAM_URL="${GRAPH_UPSTREAM_URL:-$BACKEND_UPSTREAM_URL}"
GRAPH_UPSTREAM_URL="${GRAPH_UPSTREAM_URL%/}"
NEWS_UPSTREAM_URL="${NEWS_UPSTREAM_URL:-$BACKEND_UPSTREAM_URL}"
NEWS_UPSTREAM_URL="${NEWS_UPSTREAM_URL%/}"

INDEX_HTML="/usr/share/nginx/html/index.html"

# Build runtime env JSON for client-side config (public keys only)
RUNTIME_ENV="{}"
if [ -n "${VITE_POSTHOG_API_KEY:-}" ]; then
    RUNTIME_ENV=$(printf '{"VITE_POSTHOG_API_KEY":"%s","VITE_POSTHOG_HOST":"%s"}' \
        "$VITE_POSTHOG_API_KEY" \
        "${VITE_POSTHOG_HOST:-https://t.nodrakeinthe.house}")
fi

if [ -f "$INDEX_HTML" ]; then
    tmp_index="$(mktemp)"
    sed \
        -e "s|/global.css[^\"']*|/global.css?v=${ASSET_VERSION}|g" \
        -e "s|/build/bundle.css[^\"']*|/build/bundle.css?v=${ASSET_VERSION}|g" \
        -e "s|/build/bundle.js[^\"']*|/build/bundle.js?v=${ASSET_VERSION}|g" \
        -e "s|</head>|<script>window.__ENV__=${RUNTIME_ENV};</script></head>|" \
        "$INDEX_HTML" > "$tmp_index"
    mv "$tmp_index" "$INDEX_HTML"
    chmod 644 "$INDEX_HTML"
fi

export BACKEND_UPSTREAM_URL ANALYTICS_UPSTREAM_URL GRAPH_UPSTREAM_URL NEWS_UPSTREAM_URL PORT DNS_RESOLVER ASSET_VERSION

envsubst '${BACKEND_UPSTREAM_URL} ${ANALYTICS_UPSTREAM_URL} ${GRAPH_UPSTREAM_URL} ${NEWS_UPSTREAM_URL} ${PORT} ${DNS_RESOLVER}' \
    < /etc/nginx/templates/default.conf.template \
    > /etc/nginx/conf.d/default.conf

exec nginx -g 'daemon off;'
