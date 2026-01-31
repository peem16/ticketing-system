-- Initialize database extensions and settings
-- This script runs when PostgreSQL container starts for the first time

-- Enable UUID extension (useful for generating UUIDs in database)
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Create additional databases if needed for other services
-- CREATE DATABASE ticket_service;
-- CREATE DATABASE notification_service;

-- Grant privileges
GRANT ALL PRIVILEGES ON DATABASE auth_service TO postgres;
