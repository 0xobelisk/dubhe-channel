# Dubhe Channel Configuration Guide

## 📄 Configuration Overview

`config.toml` is the main configuration file for Dubhe Channel nodes, containing all module configuration options. The file is located in the project root directory and will automatically generate default configurations on first run.

## 🚀 Quick Start

### 1. Configuration Validation

Before starting the node, it's recommended to validate the configuration file format:

```bash
# Validate config.toml in root directory
cargo run --bin config-validator

# Validate specific configuration file
cargo run --bin config-validator -- custom-config.toml
```

### 2. Starting the Node

```bash
# Use default config.toml
cargo run --bin dubhe-node

# Use specific configuration file
cargo run --bin dubhe-node -- --config custom-config.toml
```

## 📋 Configuration File Structure

### Core Service Configuration

#### API Service Configuration

```toml
[api]
rpc_bind = "127.0.0.1:8545"      # JSON-RPC service address
grpc_bind = "127.0.0.1:9090"     # gRPC service address
ws_bind = "127.0.0.1:8546"       # WebSocket service address
max_connections = 1000           # Maximum connections
request_timeout_ms = 30000       # Request timeout
```

#### Blockchain Adapter Configuration

```toml
[adapters.ethereum]
rpc_url = "https://eth-mainnet.g.alchemy.com/v2/YOUR-API-KEY"
ws_url = "wss://eth-mainnet.g.alchemy.com/v2/YOUR-API-KEY"
chain_id = 1

[adapters.sui]
rpc_url = "https://fullnode.testnet.sui.io"
network_type = "Testnet"           # Mainnet | Testnet | Devnet | Localnet
package_ids = [
    "0x1",                         # Sui Framework (default)
    "0x2",                         # Sui System (default)
    # "0xYOUR_PACKAGE_ID_HERE"     # Add your package ID
]

# Other supported adapters: solana, aptos, bitcoin
```

#### Parallel Scheduler Configuration

```toml
[scheduler]
worker_threads = 8                        # Number of worker threads
batch_size = 100                         # Batch processing size
max_queue_size = 10000                   # Maximum queue length
timeout_ms = 30000                       # Timeout duration
enable_optimistic_execution = true       # Enable optimistic execution
```

#### Virtual Machine Configuration

```toml
[vm]
default_vm = "PolkaVM"    # Default VM: PolkaVM | CkbVM | Cartesi
max_instances = 100       # Maximum VM instances
```

#### Node Core Configuration

```toml
[node]
data_dir = "./data"                # Data directory
strategy = "SolanaParallel"        # Execution strategy: SolanaParallel | AptosSTM | SuiObject
enable_metrics = true              # Enable metrics collection
```

### Optional Extended Configuration

#### Security Configuration

```toml
[security]
enable_tee = false                 # Enable Trusted Execution Environment
enable_sgx = false                 # Enable Intel SGX
enable_access_control = false      # Enable access control
audit_level = "Basic"              # Audit level: None | Basic | Detailed
```

#### Observability Configuration

```toml
[observability]
enable_prometheus = true           # Enable Prometheus metrics
prometheus_port = 9100            # Prometheus port
enable_tracing = true             # Enable distributed tracing
jaeger_endpoint = "http://localhost:14268/api/traces"
log_level = "info"                # Log level
structured_logging = true         # Structured logging
```

#### Cache Configuration

```toml
[cache]
cache_dir = "./cache"             # Cache directory
memory_cache_size = 1000          # Memory cache size
enable_compression = true         # Enable compression
cleanup_interval_hours = 24       # Cleanup interval
```

#### Performance Tuning Configuration

```toml
[performance]
tokio_worker_threads = 0          # Tokio worker threads (0=auto)
enable_numa_affinity = false     # NUMA affinity
memory_pool_size_mb = 512         # Memory pool size
gc_threshold_mb = 256             # GC trigger threshold
enable_cpu_affinity = false      # CPU affinity
```

#### Monitoring and Alerting Configuration

```toml
[alerting]
enable_alerts = false             # Enable alerting

[alerting.email]
smtp_server = "smtp.gmail.com"
smtp_port = 587
username = "your-email@gmail.com"
password = "your-app-password"
to_addresses = ["admin@dubhe.com"]

[alerting.thresholds]
cpu_usage_percent = 80            # CPU usage alert threshold
memory_usage_percent = 85         # Memory usage alert threshold
disk_usage_percent = 90           # Disk usage alert threshold
```

## 🌐 WebSocket Optimization Configuration

### Nginx WebSocket Proxy Configuration

Create an optimized nginx configuration for WebSocket proxy:

```nginx
# /etc/nginx/nginx.conf or /www/server/nginx/conf/nginx.conf

user www;
worker_processes auto;
worker_rlimit_nofile 65535;
error_log /var/log/nginx/error.log warn;
pid /var/run/nginx.pid;

events {
    worker_connections 8192;      # Optimized connection limit
    use epoll;
    multi_accept on;
}

http {
    include /etc/nginx/mime.types;
    default_type application/octet-stream;

    # Log formats
    log_format main '$remote_addr - $remote_user [$time_local] "$request" '
                    '$status $body_bytes_sent "$http_referer" '
                    '"$http_user_agent" "$http_x_forwarded_for"';

    # WebSocket-specific log format
    log_format websocket_format '$remote_addr - $remote_user [$time_local] "$request" '
                               '$status $body_bytes_sent "$http_referer" '
                               '"$http_user_agent" "$http_x_forwarded_for" '
                               '"$http_upgrade" "$connection_upgrade" '
                               'rt=$request_time uct="$upstream_connect_time" '
                               'uht="$upstream_header_time" urt="$upstream_response_time"';

    # Connection and rate limiting
    limit_conn_zone $binary_remote_addr zone=perip:10m;
    limit_req_zone $binary_remote_addr zone=api:10m rate=2r/s;
    limit_req_zone $binary_remote_addr zone=normal:10m rate=5r/s;
    limit_req_zone $binary_remote_addr zone=websocket:10m rate=1r/s;

    # WebSocket upgrade mapping
    map $http_upgrade $connection_upgrade {
        default upgrade;
        '' close;
    }

    # Optimized upstream for Dubhe backend
    upstream dubhe_backend {
        server 43.154.98.251:9944;
        keepalive 4;                    # Minimal persistent connections
        keepalive_requests 50;          # Requests per connection
        keepalive_timeout 15s;          # Connection timeout
    }

    # Include site configurations
    include /etc/nginx/conf.d/*.conf;
}
```

### Site-Specific WebSocket Configuration

```nginx
# /etc/nginx/conf.d/dubhe-websocket.conf

server {
    listen 80;
    listen 443 ssl http2;
    server_name dubheos-node-devnet-wss.obelisk.build;

    # SSL configuration
    ssl_certificate /path/to/ssl/cert.pem;
    ssl_certificate_key /path/to/ssl/key.pem;
    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_ciphers HIGH:!aNULL:!MD5;

    # Connection limits
    limit_conn perip 5;
    limit_req zone=normal burst=10 nodelay;

    # WebSocket endpoint with optimization
    location /wss {
        # Strict rate limiting for WebSocket
        limit_req zone=websocket burst=3 nodelay;

        # Proxy configuration
        proxy_pass http://dubhe_backend;
        proxy_http_version 1.1;

        # WebSocket headers
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection $connection_upgrade;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;

        # Optimized timeouts
        proxy_connect_timeout 5s;
        proxy_send_timeout 60s;
        proxy_read_timeout 60s;

        # Buffer settings
        proxy_buffering off;
        proxy_cache_bypass $http_upgrade;

        # Error handling
        proxy_next_upstream error timeout invalid_header http_500 http_502 http_503;
        proxy_next_upstream_timeout 3s;
        proxy_next_upstream_tries 2;

        # Logging
        access_log /var/log/nginx/websocket_access.log websocket_format;
    }

    # Health check endpoint
    location /health {
        access_log off;
        proxy_pass http://dubhe_backend/health;
        proxy_connect_timeout 1s;
        proxy_read_timeout 1s;
    }

    # Status monitoring
    location /nginx_status {
        stub_status on;
        access_log off;
        allow 127.0.0.1;
        allow ::1;
        deny all;
    }
}
```

## 🔧 Sui Network and Package ID Configuration

### Sui Network Types

Supports four network types:

- **Mainnet**: Production environment mainnet
- **Testnet**: Development testing network (recommended)
- **Devnet**: Development network for latest features
- **Localnet**: Local development network

### Package ID Configuration

Configure Move packages to monitor in the `package_ids` array:

```toml
[adapters.sui]
network_type = "Testnet"
package_ids = [
    "0x1",    # Sui Framework (core framework, recommended)
    "0x2",    # Sui System (system package, recommended)
    "0x123456789abcdef...",  # Your custom package ID
]
```

### Getting Package Metadata

The adapter automatically loads standardized Move module information for all configured packages, similar to:

```typescript
// TypeScript version reference
const metadata = await suiInteractor.getNormalizedMoveModulesByPackage(
  packageId
);
```

### Running Package Metadata Loading Example

```bash
cargo run --example sui_metadata_loader
```

## 🔧 Custom Configuration Examples

### Development Environment Configuration

```toml
# config_development.toml - Optimized for local development

[api]
rpc_bind = "127.0.0.1:8545"
grpc_bind = "127.0.0.1:9090"
ws_bind = "127.0.0.1:8546"
max_connections = 100              # Lower for development
request_timeout_ms = 10000         # Shorter timeout

[adapters.sui]
rpc_url = "https://fullnode.testnet.sui.io"
network_type = "Testnet"
package_ids = ["0x1", "0x2"]       # Minimal packages

[scheduler]
worker_threads = 2                 # Lower for development machine
batch_size = 10                    # Smaller batches
enable_optimistic_execution = true

[vm]
default_vm = "PolkaVM"
max_instances = 10                 # Lower instance count

[observability]
enable_prometheus = false          # Disable in development
enable_tracing = true
log_level = "debug"                # Verbose logging
```

### Production Environment Configuration

```toml
# config_production.toml - Optimized for production deployment

[api]
rpc_bind = "0.0.0.0:8545"         # Listen on all interfaces
grpc_bind = "0.0.0.0:9090"
ws_bind = "0.0.0.0:8546"
max_connections = 10000            # High connection limit
request_timeout_ms = 30000

[adapters.ethereum]
rpc_url = "https://eth-mainnet.g.alchemy.com/v2/YOUR-PROD-API-KEY"
ws_url = "wss://eth-mainnet.g.alchemy.com/v2/YOUR-PROD-API-KEY"
chain_id = 1

[adapters.sui]
rpc_url = "https://fullnode.mainnet.sui.io"
network_type = "Mainnet"           # Production mainnet
package_ids = [
    "0x1", "0x2",                  # System packages
    "0xYOUR_PROD_PACKAGE_1",
    "0xYOUR_PROD_PACKAGE_2"
]

[scheduler]
worker_threads = 16                # Full CPU utilization
batch_size = 1000                  # Large batches
max_queue_size = 100000            # Large queue
enable_optimistic_execution = true

[vm]
default_vm = "PolkaVM"
max_instances = 1000               # High instance count

[security]
enable_access_control = true       # Enable security
audit_level = "Detailed"           # Full auditing

[observability]
enable_prometheus = true           # Enable monitoring
prometheus_port = 9100
enable_tracing = true
log_level = "info"                 # Production logging
structured_logging = true

[performance]
tokio_worker_threads = 0           # Auto-detect
enable_numa_affinity = true       # NUMA optimization
memory_pool_size_mb = 2048         # Large memory pool
gc_threshold_mb = 1024

[alerting]
enable_alerts = true               # Enable alerting
```

### WebSocket Load Testing Configuration

```toml
# config_loadtest.toml - Optimized for high-throughput WebSocket testing

[api]
rpc_bind = "0.0.0.0:8545"
grpc_bind = "0.0.0.0:9090"
ws_bind = "0.0.0.0:8546"
max_connections = 50000            # Very high limit
request_timeout_ms = 5000          # Fast timeout

[scheduler]
worker_threads = 32                # Maximum parallelism
batch_size = 5000                  # Large batches
max_queue_size = 1000000           # Massive queue

[performance]
tokio_worker_threads = 32          # Many async workers
memory_pool_size_mb = 8192         # Large memory pool
enable_cpu_affinity = true        # CPU pinning
```

## 📊 Configuration Validation and Monitoring

### Configuration Validation Commands

```bash
# Basic validation
cargo run --bin config-validator

# Detailed validation output
RUST_LOG=debug cargo run --bin config-validator

# Validate specific environment configuration
cargo run --bin config-validator -- config/production.toml
```

### Runtime Configuration Monitoring

After starting the node, you can view configuration status through:

```bash
# JSON-RPC configuration query
curl -X POST http://127.0.0.1:8545 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"dubhe_getChannelStatus","params":[],"id":1}'

# Check Prometheus metrics
curl http://127.0.0.1:9100/metrics

# WebSocket health check
curl -H "Upgrade: websocket" -H "Connection: Upgrade" \
     -H "Sec-WebSocket-Key: test" \
     http://localhost:8546/wss
```

## 🔒 Security Considerations

1. **Sensitive Information**: API keys, passwords and other sensitive info should use environment variables
2. **File Permissions**: Configuration files should have appropriate permissions (e.g., 600)
3. **Backup**: Production environment configurations should be regularly backed up
4. **Version Control**: Avoid committing configuration files with sensitive information to version control
5. **WebSocket Security**: Ensure proper SSL/TLS configuration for production WebSocket endpoints

## 🛠️ Troubleshooting

### Common Configuration Errors

#### TOML Syntax Error

```
Error: invalid type: integer `127.0.0.1`, expected a string
```

**Solution**: Ensure IP addresses are wrapped in quotes

#### Port Conflicts

```
Error: Address already in use (os error 48)
```

**Solution**: Check if ports are being used by other processes, or modify port numbers in configuration

#### Permission Errors

```
Error: Permission denied (os error 13)
```

**Solution**: Check write permissions for data directory and cache directory

#### WebSocket Connection Errors

```
Error: WebSocket connection failed: 1006 Abnormal Closure
```

**Solution**: Check backend Dubhe node health and nginx proxy configuration

### Debugging Tips

1. **Enable Verbose Logging**:

   ```bash
   RUST_LOG=debug cargo run --bin dubhe-node
   ```

2. **Validate Configuration**:

   ```bash
   cargo run --bin config-validator
   ```

3. **Check System Resources**:

   ```bash
   # Check available ports
   netstat -tulpn | grep LISTEN

   # Check disk space
   df -h

   # Check memory usage
   free -h

   # Check WebSocket connectivity
   curl -v -H "Upgrade: websocket" -H "Connection: Upgrade" \
        -H "Sec-WebSocket-Key: test" http://localhost:8546/wss
   ```

### WebSocket-Specific Troubleshooting

#### Connection Pool Saturation

```bash
# Monitor connection pool status
watch "curl -s localhost/nginx_status"

# Check backend connectivity
curl -I http://43.154.98.251:9944/health

# Monitor connection states
netstat -an | grep :9944 | awk '{print $6}' | sort | uniq -c
```

#### Rate Limiting Issues

```bash
# Check rate limit status
tail -f /var/log/nginx/error.log | grep "limiting requests"

# Monitor request patterns
tail -f /var/log/nginx/websocket_access.log
```

#### SSL/TLS Problems

```bash
# Test SSL configuration
openssl s_client -connect your-domain.com:443 -servername your-domain.com

# Verify certificate chain
nginx -T | grep ssl
```

## 📈 Performance Optimization

### WebSocket Performance Tuning

1. **Connection Pool Optimization**:

   - Set `keepalive 2-4` for minimal resource usage
   - Use `keepalive_requests 25-50` for balanced throughput
   - Configure `keepalive_timeout 10-15s` for quick cleanup

2. **Rate Limiting Strategy**:

   - WebSocket: 1 request/second strict limit
   - API: 2-5 requests/second with burst capability
   - Per-IP: 3-5 concurrent connections maximum

3. **Timeout Configuration**:

   - `proxy_connect_timeout`: 3-5 seconds
   - `proxy_send_timeout`: 30-60 seconds
   - `proxy_read_timeout`: 30-60 seconds

4. **Buffer Management**:
   - Disable `proxy_buffering` for WebSocket
   - Set appropriate `client_header_buffer_size`
   - Configure `client_body_buffer_size` for request handling

This comprehensive configuration guide ensures optimal WebSocket performance and production-ready deployment of Dubhe Channel nodes.

---

**Dubhe Channel** - Let multi-chain contracts bloom in parallel within a unified execution layer ✨
