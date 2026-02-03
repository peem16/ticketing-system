---
name: db
description: Database operations - migrations, shell, status
disable-model-invocation: true
---

# Database Operations

Available operations (ask the user which one, or infer from context):

## Run Migrations
```bash
make migrate
```

## Open Database Shell
```bash
make db-shell
```

## Check Migration Status
```bash
cd services/auth-service && diesel migration list
```

## Create New Migration
```bash
cd services/auth-service && diesel migration generate <migration_name>
```
Then edit the generated `up.sql` and `down.sql` files.

## Reset Database
```bash
make down-clean && make up-build
```
Warning: This destroys all data. Confirm with the user before proceeding.

## View Current Schema
```bash
docker-compose exec postgres psql -U postgres -d auth_service -c '\dt'
docker-compose exec postgres psql -U postgres -d auth_service -c '\d+ users'
```
