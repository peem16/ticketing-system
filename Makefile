.PHONY: up down build logs clean db-shell auth-logs gateway-logs migrate

# Start all services
up:
	docker-compose up -d

# Start with build
up-build:
	docker-compose up -d --build

# Stop all services
down:
	docker-compose down

# Stop and remove volumes
down-clean:
	docker-compose down -v

# Build images
build:
	docker-compose build

# View all logs
logs:
	docker-compose logs -f

# View auth service logs
auth-logs:
	docker-compose logs -f auth-service

# View API gateway logs
gateway-logs:
	docker-compose logs -f api-gateway

# Access PostgreSQL shell
db-shell:
	docker-compose exec postgres psql -U postgres -d auth_service

# Run migrations manually (if needed)
migrate:
	docker-compose exec auth-service diesel migration run

# Clean up everything
clean:
	docker-compose down -v --rmi all
