#!/bin/bash

PR=$1
PORT=$2

mkdir -p staging
mkdir -p data/staging/conf.d

cat <<EOF > "data/staging/conf.d/adrastos-pr-$PR.xenfo.dev.conf"
server {
  listen 80;
  listen [::]:80;
  server_name adrastos-pr-$PR.xenfo.dev;

  # reverse proxy
  location / {
    proxy_pass http://127.0.0.1:$PORT;
    proxy_set_header Host \$host;
    include nginxconfig.io/proxy.conf;
  }

  # additional config
  include nginxconfig.io/general.conf;
}
EOF

cat <<EOF > "staging/docker-compose.pr-$PR.yml"
version: '3.9'

services:
  emails:
    image: ghcr.io/xenfo/adrastos-emails:pr-$PR
    pull_policy: always
    restart: unless-stopped
    env_file:
      - staging.env
    networks:
      - adrastos_default

  app:
    image: ghcr.io/xenfo/adrastos-app:pr-$PR
    pull_policy: always
    restart: unless-stopped
    depends_on:
      - emails
    env_file:
      - staging.env
    networks:
      - adrastos_default
    ports:
      - $PORT:8000
    volumes:
      - ~/.postgresql/root.crt:/work/certs/cockroach.crt
EOF

docker compose -f "staging/docker-compose.pr-$PR.yml" up -d
docker exec adrastos-staging-nginx-1 /usr/sbin/nginx -s reload
