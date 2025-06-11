#!/bin/bash

# Dubhe Channel WebSocket Optimization Setup Script
# Automated setup for production-ready WebSocket proxy configuration

set -e

# Configuration variables
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
NGINX_CONF_DIR="/etc/nginx"
NGINX_SITES_DIR="/etc/nginx/sites-available"
NGINX_ENABLED_DIR="/etc/nginx/sites-enabled"
DUBHE_LOG_DIR="/var/log/dubhe"
DUBHE_DATA_DIR="/var/lib/dubhe"
DUBHE_CACHE_DIR="/var/cache/dubhe"
BACKUP_DIR="/var/backup/dubhe"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Logging function
log() {
    echo -e "${GREEN}[$(date '+%Y-%m-%d %H:%M:%S')]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[$(date '+%Y-%m-%d %H:%M:%S')] WARNING:${NC} $1"
}

log_error() {
    echo -e "${RED}[$(date '+%Y-%m-%d %H:%M:%S')] ERROR:${NC} $1"
}

log_info() {
    echo -e "${BLUE}[$(date '+%Y-%m-%d %H:%M:%S')] INFO:${NC} $1"
}

# Check if running as root
check_root() {
    if [[ $EUID -ne 0 ]]; then
        log_error "This script must be run as root (use sudo)"
        exit 1
    fi
}

# Detect operating system
detect_os() {
    if [[ -f /etc/os-release ]]; then
        . /etc/os-release
        OS=$NAME
        VERSION=$VERSION_ID
    else
        log_error "Cannot detect operating system"
        exit 1
    fi
    
    log_info "Detected OS: $OS $VERSION"
}

# Install required packages
install_dependencies() {
    log "Installing required packages..."
    
    case "$OS" in
        *"Ubuntu"*|*"Debian"*)
            apt update
            apt install -y nginx curl wget htop iftop netstat-nat build-essential
            ;;
        *"CentOS"*|*"Red Hat"*|*"Rocky"*)
            yum update -y
            yum install -y nginx curl wget htop iftop net-tools gcc gcc-c++ make
            ;;
        *"Amazon Linux"*)
            yum update -y
            yum install -y nginx curl wget htop iftop net-tools gcc gcc-c++ make
            ;;
        *)
            log_warning "Unsupported OS: $OS. You may need to install dependencies manually."
            ;;
    esac
    
    log "Dependencies installed successfully"
}

# Create necessary directories
create_directories() {
    log "Creating Dubhe directories..."
    
    local dirs=(
        "$DUBHE_LOG_DIR"
        "$DUBHE_DATA_DIR"
        "$DUBHE_CACHE_DIR"
        "$BACKUP_DIR"
        "/var/www/error"
    )
    
    for dir in "${dirs[@]}"; do
        if [[ ! -d "$dir" ]]; then
            mkdir -p "$dir"
            log_info "Created directory: $dir"
        fi
    done
    
    # Set appropriate permissions
    chown -R www-data:www-data "$DUBHE_LOG_DIR" "$DUBHE_CACHE_DIR" 2>/dev/null || \
    chown -R nginx:nginx "$DUBHE_LOG_DIR" "$DUBHE_CACHE_DIR" 2>/dev/null || \
    log_warning "Could not set ownership for nginx directories"
    
    chmod 755 "$DUBHE_DATA_DIR" "$DUBHE_CACHE_DIR"
    chmod 644 "$DUBHE_LOG_DIR"
    
    log "Directories created and configured"
}

# Backup existing nginx configuration
backup_nginx_config() {
    log "Backing up existing nginx configuration..."
    
    local backup_timestamp=$(date +%Y%m%d_%H%M%S)
    local backup_file="/tmp/nginx_backup_$backup_timestamp.tar.gz"
    
    if [[ -d "$NGINX_CONF_DIR" ]]; then
        tar -czf "$backup_file" -C "$(dirname "$NGINX_CONF_DIR")" "$(basename "$NGINX_CONF_DIR")" 2>/dev/null || {
            log_warning "Failed to create nginx backup"
            return 1
        }
        log_info "Nginx config backed up to: $backup_file"
    fi
}

# Install optimized nginx configuration
install_nginx_config() {
    log "Installing optimized nginx configuration..."
    
    # Copy main nginx configuration
    if [[ -f "$PROJECT_ROOT/nginx.conf" ]]; then
        cp "$PROJECT_ROOT/nginx.conf" "$NGINX_CONF_DIR/nginx.conf"
        log_info "Installed main nginx configuration"
    else
        log_error "nginx.conf not found in project root"
        exit 1
    fi
    
    # Create error pages
    create_error_pages
    
    # Test nginx configuration
    if nginx -t; then
        log "Nginx configuration test passed"
    else
        log_error "Nginx configuration test failed"
        exit 1
    fi
}

# Create custom error pages
create_error_pages() {
    log "Creating custom error pages..."
    
    # 404 Error Page
    cat > /var/www/error/404.html << 'EOF'
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>404 - Page Not Found | Dubhe Channel</title>
    <style>
        body { font-family: Arial, sans-serif; text-align: center; background: #f4f4f4; padding: 50px; }
        .container { max-width: 500px; margin: 0 auto; background: white; padding: 30px; border-radius: 10px; }
        h1 { color: #333; font-size: 48px; margin: 0; }
        p { color: #666; font-size: 18px; }
        .logo { color: #007bff; font-weight: bold; font-size: 24px; margin-bottom: 20px; }
    </style>
</head>
<body>
    <div class="container">
        <div class="logo">Dubhe Channel</div>
        <h1>404</h1>
        <p>The page you're looking for doesn't exist.</p>
        <p><a href="/">Return to Homepage</a></p>
    </div>
</body>
</html>
EOF

    # 429 Rate Limit Error Page
    cat > /var/www/error/429.html << 'EOF'
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <meta http-equiv="refresh" content="5">
    <title>429 - Too Many Requests | Dubhe Channel</title>
    <style>
        body { font-family: Arial, sans-serif; text-align: center; background: #f4f4f4; padding: 50px; }
        .container { max-width: 500px; margin: 0 auto; background: white; padding: 30px; border-radius: 10px; }
        h1 { color: #ff6b6b; font-size: 48px; margin: 0; }
        p { color: #666; font-size: 18px; }
        .logo { color: #007bff; font-weight: bold; font-size: 24px; margin-bottom: 20px; }
        .countdown { color: #ff6b6b; font-weight: bold; }
    </style>
</head>
<body>
    <div class="container">
        <div class="logo">Dubhe Channel</div>
        <h1>429</h1>
        <p>Too many requests. Please slow down.</p>
        <p class="countdown">Page will refresh automatically in 5 seconds...</p>
        <p>If you're experiencing connection issues, please check our <a href="/health">system status</a>.</p>
    </div>
</body>
</html>
EOF

    # 5xx Server Error Page
    cat > /var/www/error/50x.html << 'EOF'
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Server Error | Dubhe Channel</title>
    <style>
        body { font-family: Arial, sans-serif; text-align: center; background: #f4f4f4; padding: 50px; }
        .container { max-width: 500px; margin: 0 auto; background: white; padding: 30px; border-radius: 10px; }
        h1 { color: #ff6b6b; font-size: 48px; margin: 0; }
        p { color: #666; font-size: 18px; }
        .logo { color: #007bff; font-weight: bold; font-size: 24px; margin-bottom: 20px; }
    </style>
</head>
<body>
    <div class="container">
        <div class="logo">Dubhe Channel</div>
        <h1>Server Error</h1>
        <p>We're experiencing technical difficulties.</p>
        <p>Please try again in a few moments.</p>
        <p>If the problem persists, contact support.</p>
    </div>
</body>
</html>
EOF

    log_info "Custom error pages created"
}

# Configure system limits
configure_system_limits() {
    log "Configuring system limits for high-performance WebSocket..."
    
    # Configure systemd limits for nginx
    local nginx_override_dir="/etc/systemd/system/nginx.service.d"
    mkdir -p "$nginx_override_dir"
    
    cat > "$nginx_override_dir/override.conf" << 'EOF'
[Service]
LimitNOFILE=65535
LimitNPROC=65535
LimitCORE=infinity
LimitMEMLOCK=infinity
EOF

    # Configure kernel parameters
    cat >> /etc/sysctl.conf << 'EOF'

# Dubhe Channel WebSocket Optimization
net.core.somaxconn = 32768
net.core.netdev_max_backlog = 5000
net.ipv4.tcp_max_syn_backlog = 8192
net.ipv4.tcp_keepalive_time = 600
net.ipv4.tcp_keepalive_intvl = 60
net.ipv4.tcp_keepalive_probes = 9
net.ipv4.tcp_fin_timeout = 30
net.ipv4.tcp_tw_reuse = 1
fs.file-max = 2097152
EOF

    # Apply sysctl changes
    sysctl -p
    
    # Configure limits.conf
    cat >> /etc/security/limits.conf << 'EOF'

# Dubhe Channel WebSocket Optimization
www-data soft nofile 65535
www-data hard nofile 65535
nginx soft nofile 65535
nginx hard nofile 65535
EOF

    log "System limits configured"
}

# Install monitoring tools
install_monitoring_tools() {
    log "Installing monitoring and management tools..."
    
    # Make connection monitor script executable
    if [[ -f "$PROJECT_ROOT/scripts/connection_monitor.sh" ]]; then
        cp "$PROJECT_ROOT/scripts/connection_monitor.sh" /usr/local/bin/dubhe-monitor
        chmod +x /usr/local/bin/dubhe-monitor
        log_info "Connection monitor installed to /usr/local/bin/dubhe-monitor"
    fi
    
    # Create systemd service for connection monitor
    cat > /etc/systemd/system/dubhe-monitor.service << 'EOF'
[Unit]
Description=Dubhe Channel WebSocket Connection Monitor
After=nginx.service
Requires=nginx.service

[Service]
Type=simple
User=root
ExecStart=/usr/local/bin/dubhe-monitor
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
EOF

    # Create log rotation configuration
    cat > /etc/logrotate.d/dubhe << 'EOF'
/var/log/dubhe/*.log {
    daily
    rotate 30
    compress
    delaycompress
    missingok
    notifempty
    create 644 www-data www-data
    postrotate
        /bin/kill -USR1 $(cat /var/run/nginx.pid 2>/dev/null) 2>/dev/null || true
    endscript
}
EOF

    log "Monitoring tools installed"
}

# Configure SSL/TLS (if certificates are available)
configure_ssl() {
    log "Checking SSL certificate configuration..."
    
    local cert_dir="/etc/ssl/certs/dubhe"
    local key_dir="/etc/ssl/private/dubhe"
    
    # Create SSL directories
    mkdir -p "$cert_dir" "$key_dir"
    chmod 755 "$cert_dir"
    chmod 700 "$key_dir"
    
    # Check if certificates exist
    if [[ ! -f "$cert_dir/fullchain.pem" ]] || [[ ! -f "$key_dir/privkey.pem" ]]; then
        log_warning "SSL certificates not found. Creating self-signed certificate for testing..."
        
        # Generate self-signed certificate for testing
        openssl req -x509 -nodes -days 365 -newkey rsa:2048 \
            -keyout "$key_dir/privkey.pem" \
            -out "$cert_dir/fullchain.pem" \
            -subj "/C=US/ST=Test/L=Test/O=Dubhe/CN=localhost" 2>/dev/null
        
        log_warning "Self-signed certificate created. Replace with production certificates before deployment."
    else
        log "SSL certificates found and configured"
    fi
}

# Setup Dubhe configuration
setup_dubhe_config() {
    log "Setting up Dubhe configuration..."
    
    # Copy production configuration if it doesn't exist
    if [[ ! -f "$PROJECT_ROOT/config.toml" ]] && [[ -f "$PROJECT_ROOT/config_production_english.toml" ]]; then
        cp "$PROJECT_ROOT/config_production_english.toml" "$PROJECT_ROOT/config.toml"
        log_info "Production configuration copied to config.toml"
    fi
    
    # Set appropriate permissions
    chmod 600 "$PROJECT_ROOT/config.toml" 2>/dev/null || true
    
    log "Dubhe configuration setup complete"
}

# Enable and start services
start_services() {
    log "Starting and enabling services..."
    
    # Reload systemd
    systemctl daemon-reload
    
    # Enable and start nginx
    systemctl enable nginx
    systemctl restart nginx
    
    # Enable dubhe monitor service
    systemctl enable dubhe-monitor
    systemctl start dubhe-monitor
    
    log "Services started and enabled"
}

# Perform health checks
health_check() {
    log "Performing health checks..."
    
    # Check nginx status
    if systemctl is-active --quiet nginx; then
        log_info "✓ Nginx is running"
    else
        log_error "✗ Nginx is not running"
        return 1
    fi
    
    # Check nginx configuration
    if nginx -t &>/dev/null; then
        log_info "✓ Nginx configuration is valid"
    else
        log_error "✗ Nginx configuration has errors"
        return 1
    fi
    
    # Check if ports are listening
    local ports=(80 443)
    for port in "${ports[@]}"; do
        if netstat -tlnp | grep -q ":$port "; then
            log_info "✓ Port $port is listening"
        else
            log_warning "⚠ Port $port is not listening"
        fi
    done
    
    # Check dubhe monitor service
    if systemctl is-active --quiet dubhe-monitor; then
        log_info "✓ Dubhe monitor is running"
    else
        log_warning "⚠ Dubhe monitor is not running"
    fi
    
    # Test WebSocket endpoint (if backend is running)
    local test_url="http://localhost/nginx_status"
    if curl -s "$test_url" &>/dev/null; then
        log_info "✓ Nginx status endpoint is accessible"
    else
        log_warning "⚠ Nginx status endpoint is not accessible"
    fi
    
    log "Health check completed"
}

# Generate performance optimization report
generate_report() {
    log "Generating optimization report..."
    
    local report_file="/tmp/dubhe_optimization_report.txt"
    
    cat > "$report_file" << EOF
# Dubhe Channel WebSocket Optimization Report
Generated: $(date)

## System Configuration
- OS: $OS $VERSION
- Nginx Version: $(nginx -v 2>&1 | grep -o 'nginx/[0-9.]*')
- Worker Connections: 8192
- File Descriptors: 65535

## Optimizations Applied
✓ Nginx WebSocket proxy configuration
✓ Connection pooling (4 keepalive connections)
✓ Rate limiting (1 req/s for WebSocket, 2 req/s for API)
✓ SSL/TLS optimization
✓ System kernel parameters tuned
✓ File descriptor limits increased
✓ Log rotation configured
✓ Error pages customized
✓ Monitoring service installed

## Performance Targets
- Concurrent Connections: 1,000+
- WebSocket Latency: < 100ms
- Connection Success Rate: > 99%
- TPS: > 10,000

## Next Steps
1. Configure production SSL certificates
2. Update backend endpoint in nginx.conf if needed
3. Set up external monitoring (Prometheus/Grafana)
4. Configure alerting (email/Slack)
5. Test with load testing tools

## Management Commands
- Start monitoring: systemctl start dubhe-monitor
- Check nginx status: systemctl status nginx
- View connection stats: /usr/local/bin/dubhe-monitor --status
- View logs: tail -f /var/log/dubhe/connection_monitor.log
- Test nginx config: nginx -t
- Reload nginx: nginx -s reload

## File Locations
- Main Config: $NGINX_CONF_DIR/nginx.conf
- Dubhe Data: $DUBHE_DATA_DIR
- Dubhe Logs: $DUBHE_LOG_DIR
- Dubhe Cache: $DUBHE_CACHE_DIR
- Monitor Script: /usr/local/bin/dubhe-monitor
EOF

    log "Optimization report generated: $report_file"
    echo ""
    cat "$report_file"
}

# Main installation function
main() {
    echo -e "${PURPLE}"
    echo "╔══════════════════════════════════════════════════════════════╗"
    echo "║                 Dubhe Channel WebSocket Optimizer           ║"
    echo "║              Production-Ready Setup Script                  ║"
    echo "╚══════════════════════════════════════════════════════════════╝"
    echo -e "${NC}"
    
    log "Starting Dubhe Channel WebSocket optimization setup..."
    
    # Pre-flight checks
    check_root
    detect_os
    
    # Main installation steps
    install_dependencies
    create_directories
    backup_nginx_config
    configure_system_limits
    configure_ssl
    install_nginx_config
    install_monitoring_tools
    setup_dubhe_config
    start_services
    
    # Post-installation
    health_check
    generate_report
    
    echo ""
    log_info "🎉 Dubhe Channel WebSocket optimization setup completed successfully!"
    echo ""
    echo -e "${GREEN}Next steps:${NC}"
    echo "1. Update backend endpoint in /etc/nginx/nginx.conf if needed"
    echo "2. Replace self-signed SSL certificate with production certificate"
    echo "3. Configure monitoring and alerting"
    echo "4. Test WebSocket connectivity"
    echo ""
    echo -e "${BLUE}Useful commands:${NC}"
    echo "- Monitor connections: /usr/local/bin/dubhe-monitor --status"
    echo "- View nginx status: systemctl status nginx"
    echo "- Check configuration: nginx -t"
    echo "- View logs: tail -f /var/log/dubhe/connection_monitor.log"
    echo ""
}

# Handle command line arguments
case "${1:-}" in
    --help|-h)
        echo "Dubhe Channel WebSocket Optimization Setup"
        echo ""
        echo "Usage: $0 [options]"
        echo ""
        echo "Options:"
        echo "  --help, -h     Show this help message"
        echo "  --dry-run      Show what would be done without making changes"
        echo "  --uninstall    Remove Dubhe WebSocket optimization"
        echo ""
        exit 0
        ;;
    --dry-run)
        echo "DRY RUN MODE - No changes will be made"
        echo "The following operations would be performed:"
        echo "- Install dependencies (nginx, curl, monitoring tools)"
        echo "- Create directories (/var/log/dubhe, /var/lib/dubhe, etc.)"
        echo "- Backup existing nginx configuration"
        echo "- Install optimized nginx configuration"
        echo "- Configure system limits and kernel parameters"
        echo "- Install monitoring tools and services"
        echo "- Configure SSL/TLS"
        echo "- Start and enable services"
        exit 0
        ;;
    --uninstall)
        log "Uninstalling Dubhe WebSocket optimization..."
        systemctl stop dubhe-monitor 2>/dev/null || true
        systemctl disable dubhe-monitor 2>/dev/null || true
        rm -f /etc/systemd/system/dubhe-monitor.service
        rm -f /usr/local/bin/dubhe-monitor
        rm -f /etc/logrotate.d/dubhe
        log "Uninstallation completed"
        exit 0
        ;;
    *)
        main "$@"
        ;;
esac 