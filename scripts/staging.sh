#!/bin/bash

case $1 in
cou)
  PR=$2
  BRANCH=$3

  mkdir -p staging
  mkdir -p data/nginx/staging/conf.d

  if [ ! -f "staging/docker-compose.pr-$PR.yml" ]; then
    cat <<EOF >"data/nginx/staging/conf.d/adrastos-api-pr-$PR.xenfo.dev.conf"
server {
  listen 80;
  listen [::]:80;
  server_name adrastos-api-pr-$PR.xenfo.dev;

  # reverse proxy
  location / {
    proxy_pass http://adrastos-staging-pr-$PR-app-1.adrastos_default:8000;
    proxy_set_header Host \$host;
    include nginxconfig.io/proxy.conf;
  }

  # additional config
  include nginxconfig.io/general.conf;
}
EOF

    cat <<EOF >"staging/docker-compose.pr-$PR.yml"
name: adrastos-staging-pr-$PR
version: '3.9'

services:
  emails:
    image: ghcr.io/xenfo/adrastos-emails:staging-pr-$PR
    pull_policy: always
    restart: unless-stopped
    env_file:
      - ../staging.env

  app:
    image: ghcr.io/xenfo/adrastos-app:staging-pr-$PR
    pull_policy: always
    restart: unless-stopped
    depends_on:
      - emails
    env_file:
      - ../staging.env
    environment:
      - CLIENT_URL=https://adrastos-git-$(echo "$BRANCH" | sed 's/\//-/')-xenfo.vercel.app
    networks:
      - adrastos_default
    volumes:
      - ~/.postgresql/root.crt:/work/certs/cockroach.crt

networks:
  adrastos_default:
    external: true
EOF
  fi

  su -- sam -c "docker compose -f \"staging/docker-compose.pr-$PR.yml\" up -d"
  docker compose --profile deploy restart staging-nginx
  docker exec adrastos-nginx-1 /usr/sbin/nginx -s reload
  ;;
destroy)
  PR=$2

  docker compose -f "staging/docker-compose.pr-$PR.yml" down -v
  rm -rf "staging/docker-compose.pr-$PR.yml"
  rm -rf "data/nginx/staging/conf.d/adrastos-api-pr-$PR.xenfo.dev.conf"
  docker compose --profile deploy restart staging-nginx
  docker rmi $(docker images -q --filter "reference=ghcr.io/xenfo/adrastos-*:*")

  exit 0
  ;;
esac
