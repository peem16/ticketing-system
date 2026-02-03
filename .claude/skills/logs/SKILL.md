---
name: logs
description: View and analyze service logs
disable-model-invocation: true
---

# Service Logs

Available operations:

## All Services
```bash
make logs
```

## Auth Service Only
```bash
make auth-logs
```

## API Gateway Only
```bash
make gateway-logs
```

## Recent Logs (last 50 lines)
```bash
docker-compose logs --tail=50 auth-service
docker-compose logs --tail=50 api-gateway
```

## Filter for Errors
```bash
docker-compose logs auth-service 2>&1 | grep -i "error\|panic\|fatal"
docker-compose logs api-gateway 2>&1 | grep -i "error\|panic\|fatal"
```

## Kafka Logs
```bash
docker-compose logs kafka
```

## PostgreSQL Logs
```bash
docker-compose logs postgres
```

After viewing logs, summarize any errors or warnings found.
