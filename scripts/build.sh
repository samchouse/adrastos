#!/usr/bin/env bash

docker build -t ghcr.io/xenfo/adrastos-app:latest -f crates/app/Dockerfile .
docker build -t ghcr.io/xenfo/adrastos-emails:latest -f packages/emails/Dockerfile .
