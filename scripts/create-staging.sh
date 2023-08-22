#!/bin/bash

PR=$1
PORT=$2

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
