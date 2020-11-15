#!/bin/bash

docker run -d \
    --name rust-postgres \
    -e POSTGRES_PASSWORD=Asdfg12345qwert \
    -e PGDATA=/var/lib/postgresql/data/pgdata \
    -p 5432:5432/tcp \
    -v /Users/maxkul/Code/hacker-news/postgres/pgdata:/var/lib/postgresql/data \
    postgres
