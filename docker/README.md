# 🦎 Komodo Docker Deployment

This directory contains Docker and Docker Compose files for deploying Komodo Client (Agent) and Server (Periphery) in containerized environments.

## 📁 Files

- `Dockerfile.client` - Dockerfile for Komodo Client (Agent)
- `Dockerfile.server` - Dockerfile for Komodo Server (Periphery)
- `docker-compose.yml` - Docker Compose file for both services
- `env.example` - Example environment variables file
- `README.md` - This file

## 🚀 Quick Start

### 1. Configure environment variables

```bash
cd docker
cp env.example .env
# Edit .env and adjust the values to your environment
```

### 2. Create directories

```bash
# Create the root directory for Periphery
mkdir -p ../komodo-root/{stacks,repos,builds}

# Create the workspace directory for the client (optional)
mkdir -p ../workspace
```

### 3. Start containers

```bash
# Start both services
docker-compose up -d

# Start only the server
docker-compose up -d komodo-server

# Start only the client
docker-compose up -d komodo-client
```

### 4. View logs

```bash
# All logs
docker-compose logs -f

# Server logs only
docker-compose logs -f komodo-server

# Client logs only
docker-compose logs -f komodo-client
```

## ⚙️ Configuration

### Environment Variables

Both services can be configured via environment variables. The complete list can be found in:

- **Client**: [komodo.cli.toml](../config/komodo.cli.toml)
- **Server**: [periphery.config.toml](../config/periphery.config.toml)

### Configuration Files

Alternatively, you can use configuration files:

#### Client Configuration

Create a `komodo.cli.toml` file and mount it:

```yaml
volumes:
  - ./config/komodo.cli.toml:/config/komodo.cli.toml
```

#### Server Configuration

Create a `periphery.config.toml` file and mount it:

```yaml
volumes:
  - ./config/periphery.config.toml:/config/periphery.config.toml
```

## 🔐 Authentication

### Server (Periphery)

The server requires keys for authentication:

1. **Private Key**: Automatically generated if not present
   - Default path: `/config/keys/periphery.key`

2. **Core Public Key**: Required for outbound mode
   - Default path: `/config/keys/core.pub`
   - Must be provided by Komodo Core

### Client (Agent)

The client requires API credentials from Komodo Core:

1. Create an API key in the Komodo Core UI
2. Set the environment variables:
   ```bash
   KOMODO_CLI_KEY=K-...
   KOMODO_CLI_SECRET=S-...
   ```

## 🌐 Network Configuration

### Server Port

The server listens on port `8120` by default. You can change the host port via environment variable:

```bash
PERIPHERY_HOST_PORT=8120
```

### Network Modes

#### Outbound Mode (Server connects to Core)

```bash
PERIPHERY_CORE_ADDRESS=ws://komodo-core:9120
PERIPHERY_CONNECT_AS=server-name
PERIPHERY_CORE_PUBLIC_KEYS=file:/config/keys/core.pub
```

#### Inbound Mode (Core connects to Server)

```bash
PERIPHERY_SERVER_ENABLED=true
PERIPHERY_PORT=8120
PERIPHERY_BIND_IP=[::]
```

## 📦 Volumes

### Client Volumes

- `client-config` - Configuration files
- `client-backups` - Database backups

### Server Volumes

- `server-config` - Configuration files
- `server-keys` - Private/Public keys
- `server-ssl` - SSL certificates
- `/var/run/docker.sock` - Docker socket (for container management)
- `/proc` - System information
- `PERIPHERY_ROOT_DIRECTORY` - Root directory for stacks/repos

## 🔧 Advanced Usage

### Execute Client Commands

```bash
# Create backup
docker-compose exec komodo-client km backup database

# Perform restore
docker-compose exec komodo-client km restore database

# Other commands
docker-compose exec komodo-client km <command>
```

### Update Server Configuration

```bash
# Edit configuration file
docker-compose exec komodo-server vi /config/periphery.config.toml

# Restart server
docker-compose restart komodo-server
```

### Health Checks

Both services have health checks configured:

```bash
# Check status
docker-compose ps

# Health check details
docker inspect komodo-server | jq '.[0].State.Health'
```

## 🐛 Troubleshooting

### Server won't start

1. Check the logs:
   ```bash
   docker-compose logs komodo-server
   ```

2. Ensure Docker socket is mounted:
   ```yaml
   volumes:
     - /var/run/docker.sock:/var/run/docker.sock:ro
   ```

3. Check permissions for `/proc`:
   ```yaml
   volumes:
     - /proc:/proc:ro
   ```

### Client cannot connect

1. Check the Core address:
   ```bash
   echo $KOMODO_CLI_HOST
   ```

2. Ensure API credentials are set:
   ```bash
   echo $KOMODO_CLI_KEY
   echo $KOMODO_CLI_SECRET
   ```

3. Test the connection:
   ```bash
   docker-compose exec komodo-client km --version
   ```

### Key Issues

If the server cannot authenticate:

1. Check if keys exist:
   ```bash
   docker-compose exec komodo-server ls -la /config/keys/
   ```

2. Ensure the Core public key is correct:
   ```bash
   docker-compose exec komodo-server cat /config/keys/core.pub
   ```

## 📚 Additional Information

- [Komodo Documentation](https://komo.do/docs)
- [Client Configuration](../config/komodo.cli.toml)
- [Server Configuration](../config/periphery.config.toml)
- [GitHub Repository](https://github.com/moghtech/komodo)

## 📝 License

GPL-3.0
