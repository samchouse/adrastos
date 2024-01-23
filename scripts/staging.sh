#!/bin/bash

case $1 in
cou)
  PR=$2
  BRANCH=$3

  mkdir -p staging

  if [ ! -f "staging/docker-compose.pr-$PR.yml" ]; then
    cat <<EOF >>../../Caddyfile

adrastos-api-pr-$PR.xenfo.dev {
	reverse_proxy adrastos-staging-pr-$PR-app-1:8000
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
      - deployments_default
    volumes:
      - ~/.postgresql/root.crt:/work/certs/cockroach.crt

networks:
  deployments_default:
    external: true
EOF

    cat <<EOF >/home/sam/.cloudflared/config.yml
$(head -n-1 /home/sam/.cloudflared/config.yml)
  - hostname: adrastos-api-pr-$PR.xenfo.dev
    service: https://localhost
    originRequest:
      originServerName: adrastos-api-pr-$PR.xenfo.dev
      httpHostHeader: adrastos-api-pr-$PR.xenfo.dev
$(tail -n1 /home/sam/.cloudflared/config.yml)
EOF

    systemctl restart cloudflared
    su -- sam -c "cloudflared tunnel route dns '7d6af8ba-5ea2-4136-b245-27b513646807' \"adrastos-api-pr-$PR\""
  fi

  su -- sam -c "docker compose -f \"staging/docker-compose.pr-$PR.yml\" up -d"
  docker compose -f ../../docker-compose.yml exec -w /etc/caddy caddy caddy reload
  ;;
destroy)
  PR=$2

  yq -yi 'del(.ingress[] | select(.hostname == "adrastos-api-pr-29.xenfo.dev"))' /home/sam/.cloudflared/config.yml

  sed -i ":a;N;\$!ba; s/adrastos-api-pr-$PR\.xenfo\.dev {[^{}]*}//g" ../../Caddyfile
  sed -i -e :a -e '/^\n*$/{$d;N;};/\n$/ba' ../../Caddyfile

  docker compose -f "staging/docker-compose.pr-$PR.yml" down -v
  rm -rf "staging/docker-compose.pr-$PR.yml"
  docker compose -f ../../docker-compose.yml exec -w /etc/caddy caddy caddy reload
  docker rmi $(docker images -q --filter "reference=ghcr.io/xenfo/adrastos-*:staging-pr-*")

  exit 0
  ;;
esac
