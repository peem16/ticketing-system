#!/bin/bash
set -e

echo "Waiting for PostgreSQL to be ready..."

# Wait for PostgreSQL
until PGPASSWORD=$POSTGRES_PASSWORD psql -h "$POSTGRES_HOST" -U "$POSTGRES_USER" -d "$POSTGRES_DB" -c '\q' 2>/dev/null; do
  echo "PostgreSQL is unavailable - sleeping"
  sleep 2
done

echo "PostgreSQL is ready!"

# Run migrations
echo "Running database migrations..."
diesel migration run

echo "Migrations completed!"

# Start the application
echo "Starting Auth Service..."
exec /app/auth-service
