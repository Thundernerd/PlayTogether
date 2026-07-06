#!/bin/sh
set -e

mkdir -p /data
chown -R appuser:appuser /data

exec gosu appuser "$@"
