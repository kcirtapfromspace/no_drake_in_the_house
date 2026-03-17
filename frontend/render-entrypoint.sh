#!/bin/sh
set -eu

: "${BACKEND_HOSTPORT:?BACKEND_HOSTPORT must be set}"
: "${PORT:=10000}"

DNS_RESOLVER="${DNS_RESOLVER:-$(awk '/^nameserver[[:space:]]+/ { print $2; exit }' /etc/resolv.conf)}"

if [ -z "$DNS_RESOLVER" ]; then
    echo "Unable to determine a DNS resolver for nginx upstream lookups" >&2
    exit 1
fi

export BACKEND_HOSTPORT PORT DNS_RESOLVER

envsubst '${BACKEND_HOSTPORT} ${PORT} ${DNS_RESOLVER}' \
    < /etc/nginx/templates/default.conf.template \
    > /etc/nginx/conf.d/default.conf

exec nginx -g 'daemon off;'
