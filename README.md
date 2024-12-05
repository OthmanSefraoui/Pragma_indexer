# Pragma Price Indexer

A Rust application that indexes price data from Pragma using Apibara, provides TWAP calculations with signed responses, and enables P2P communication between nodes.

## Features

- Indexes price data from Pragma contracts using Apibara
- Stores historical price data in Redis
- Provides REST API endpoints for TWAP queries
- Signs TWAP responses using secp256k1
- P2P communication between nodes using libp2p
- Message verification using public keys

## Requirements

### Local Development

- Rust (latest stable version)
- Redis server
- Apibara API key
- Private key for signing responses

### Docker Deployment

- Docker
- Docker Compose
- Apibara API key
- Private key for signing

## Configuration

Create a `.env` file in the project root:

```env
APIBARA_API_KEY=your_apibara_api_key_here
PRIVATE_KEY=your_private_key_here  # for signing TWAP responses
REDIS_URL=redis://localhost:6379  # Optional, defaults to this value
P2P_LISTEN_ADDR=/ip4/0.0.0.0/tcp/61234  # P2P listening address
P2P_BOOTSTRAP_PEERS=/ip4/x.x.x.x/tcp/61234  # Optional, comma-separated list of bootstrap peers
```

## Installation & Running

### Local Development

1. Run the first node (Bootstrap node):

```bash
P2P_LISTEN_ADDR=/ip4/127.0.0.1/tcp/61234 cargo run
```

4. Run additional nodes:

```bash
P2P_LISTEN_ADDR=/ip4/127.0.0.1/tcp/61235 P2P_BOOTSTRAP_PEERS=/ip4/127.0.0.1/tcp/61234 cargo run
```

### Docker Deployment

1. Build and start the containers:

```bash
docker-compose up -d
```

This will start:

- The indexer application
- Redis server
- P2P networking

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
  "twap": "10207891077717",
  "period": 100000,
  "signature": "3045022100850a7aa108cbf685e14d2b70f695fe557672e262c2ee7e3d8f85bec4cbeacb9302206c394150bb136f620758f8fbe65afa5dbd107da164e966593b3263ebeab4adc0"
}
```

## Architecture

The application consists of several components:

1. **Indexer Service**:

   - Connects to Starknet via Apibara
   - Indexes price data
   - Stores data in Redis

2. **TWAP Service**:

   - Calculates Time-Weighted Average Prices
   - Signs responses with secp256k1
   - Provides verification capabilities

3. **P2P Network**:

   - Uses libp2p for node communication
   - Implements mDNS for local peer discovery
   - Uses gossipsub for message propagation
   - Verifies messages using public keys

4. **Redis Storage**:
   - Stores historical price data
   - Enables efficient TWAP calculations
   - Maintains data consistency

## P2P Network

The P2P network enables nodes to:

1. Discover other nodes automatically on the local network
2. Connect to bootstrap nodes for network entry
3. Share TWAP updates across the network
4. Verify message authenticity using signatures

### Running Multiple Nodes

To run a network of nodes:

1. Start a bootstrap node:

```bash
RUST_LOG=debug P2P_LISTEN_ADDR=/ip4/0.0.0.0/tcp/61234 cargo run
```

2. Start additional nodes:

```bash
RUST_LOG=debug P2P_LISTEN_ADDR=/ip4/0.0.0.0/tcp/61235 P2P_BOOTSTRAP_PEERS=/ip4/bootstrap_ip/tcp/61234 cargo run
```

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
│   ├── p2p.rs         # P2P networking
│   ├── signing.rs     # Message signing
│   └── redis_client.rs # Redis interactions
├── types/             # Data structures
│   ├── mod.rs
│   └── spot_entry.rs
└── config/            # Configuration
    └── mod.rs
```

## Monitoring

- Use the `/health` endpoint to monitor application health
- Check Docker logs for debugging:

```bash
docker-compose logs -f
```

- Enable debug logging with RUST_LOG:

```bash
RUST_LOG=debug cargo run
```

## Development

To modify the configuration:

1. Update the config in `src/config/mod.rs`
2. Set environment variables or modify `.env` file
3. Rebuild and restart:

```bash
cargo run
```
