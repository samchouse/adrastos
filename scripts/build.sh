#!/usr/bin/env bash

docker build -t ghcr.io/xenfo/adrastos-app:latest -f crates/app/Dockerfile .
docker build -t ghcr.io/xenfo/adrastos-emails:latest -f packages/emails/Dockerfile .

if [ "$1" == "--push" ]; then
	docker push ghcr.io/xenfo/adrastos-app:latest
	docker push ghcr.io/xenfo/adrastos-emails:latest
fi
