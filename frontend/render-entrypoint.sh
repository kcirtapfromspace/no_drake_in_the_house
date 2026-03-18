#!/bin/sh
set -eu

: "${BACKEND_UPSTREAM_URL:?BACKEND_UPSTREAM_URL must be set}"
: "${PORT:=10000}"

DNS_RESOLVER="${DNS_RESOLVER:-$(awk '/^nameserver[[:space:]]+/ { print $2; exit }' /etc/resolv.conf)}"

if [ -z "$DNS_RESOLVER" ]; then
    echo "Unable to determine a DNS resolver for nginx upstream lookups" >&2
    exit 1
fi

BACKEND_UPSTREAM_URL="${BACKEND_UPSTREAM_URL%/}"

export BACKEND_UPSTREAM_URL PORT DNS_RESOLVER

envsubst '${BACKEND_UPSTREAM_URL} ${PORT} ${DNS_RESOLVER}' \
    < /etc/nginx/templates/default.conf.template \
    > /etc/nginx/conf.d/default.conf

exec nginx -g 'daemon off;'
