#!/bin/bash

PR=$1

docker compose -f "staging/docker-compose.pr-$PR.yml" down -v
docker exec adrastos-staging-nginx-1 /usr/sbin/nginx -s reload
