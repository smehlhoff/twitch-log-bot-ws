#!/bin/sh

set -e

psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" --dbname "$POSTGRES_DB" <<-EOSQL
  CREATE USER metabase WITH PASSWORD '${MB_DB_PASS}';
  CREATE USER n8n WITH PASSWORD '${DB_POSTGRESDB_PASSWORD}';

  CREATE DATABASE metabase;
  CREATE DATABASE n8n;

  GRANT ALL PRIVILEGES ON DATABASE metabase TO metabase;
  GRANT ALL PRIVILEGES ON DATABASE n8n TO n8n;

  \connect metabase;
  GRANT ALL PRIVILEGES ON SCHEMA public TO metabase;

  \connect n8n;
  GRANT ALL PRIVILEGES ON SCHEMA public TO n8n;
EOSQL
