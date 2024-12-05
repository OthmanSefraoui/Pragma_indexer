# Pragma Price Indexer

A Rust application that indexes price data from Pragma using Apibara and provides a REST API to query Time-Weighted Average Price (TWAP).

## Features

- Indexes price data from Pragma contracts using Apibara
- Stores historical price data in Redis
- Provides REST API endpoints to query TWAP
- Supports Docker deployment
- Health check endpoint

## Requirements

### Local Development

- Rust (latest stable version)
- Redis server
- Apibara API key

### Docker Deployment

- Docker
- Docker Compose
- Apibara API key

## Configuration

Create a `.env` file in the project root:

```env
APIBARA_API_KEY=your_apibara_api_key_here
REDIS_URL=redis://localhost:6379  # Optional, defaults to this value
```

## Installation & Running

### Local Development

1. Run the application:

```bash
cargo run
```

### Docker Deployment

1. Build and start the containers:

```bash
docker-compose up -d
```

This will start both the indexer application and Redis.

## API Endpoints

### Health Check

```bash
GET /health

# Example
curl http://localhost:3000/health
```

Response:

```json
{
  "status": "up",
  "redis_connection": true
}
```

### Get TWAP Data

```bash
GET /api/get_data?pair_id=<PAIR_ID>&period=<PERIOD>

# Example (1-hour TWAP for BTC/USD)
curl "http://localhost:3000/api/get_data?pair_id=BTC/USD&period=3600"
```

Parameters:

- `pair_id`: The trading pair (e.g., "BTC/USD")
- `period`: Time period in seconds (optional, defaults to 3600)

Response:

```json
{
  "pair_id": "BTC/USD",
  "twap": 10279274680655,
  "period": 3600
}
```

## Architecture

The application consists of three main components:

1. **Indexer Service**: Connects to Starknet via Apibara and indexes price data.
2. **Redis Storage**: Stores historical price data with timestamps.
3. **REST API**: Provides endpoints to query the stored data.

## Project Structure

```
src/
├── main.rs              # Application entry point
├── api/                 # API endpoints
│   ├── mod.rs
│   └── routes.rs
├── services/           # Core services
│   ├── mod.rs
│   ├── indexer.rs     # Apibara indexer
│   └── redis_client.rs # Redis interactions
└── types/             # Data structures
    ├── mod.rs
    └── spot_entry.rs
```

## Error Handling

- The application will automatically retry connecting to Redis on startup
- Failed indexing operations are logged but won't crash the application
- API endpoints return appropriate HTTP status codes and error messages

## Monitoring

- Use the `/health` endpoint to monitor application health
- Check Docker logs for debugging:

```bash
docker-compose logs -f app
```
