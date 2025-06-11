#!/bin/bash

# Dubhe Channel WebSocket Connection Monitor
# Real-time monitoring and automated recovery for WebSocket connections

set -e

# Configuration
NGINX_STATUS_URL="http://localhost/nginx_status"
BACKEND_HEALTH_URL="http://43.154.98.251:9944/health"
MAX_CONNECTIONS=800
RELOAD_THRESHOLD=1000
WARNING_THRESHOLD=600
LOG_FILE="/var/log/dubhe/connection_monitor.log"
EMAIL_ALERT="admin@dubhe.com"

# Colors for output
RED='\033[0;31m'
YELLOW='\033[1;33m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging function
log() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] $1" | tee -a "$LOG_FILE"
}

# Check if nginx status is available
check_nginx_status() {
    if ! curl -s "$NGINX_STATUS_URL" > /dev/null 2>&1; then
        log "ERROR: Nginx status endpoint not available at $NGINX_STATUS_URL"
        return 1
    fi
    return 0
}

# Get connection metrics
get_connection_metrics() {
    local status_output
    status_output=$(curl -s "$NGINX_STATUS_URL" 2>/dev/null)
    
    if [ $? -ne 0 ]; then
        log "ERROR: Failed to fetch nginx status"
        return 1
    fi
    
    # Parse nginx status output
    ACTIVE_CONN=$(echo "$status_output" | grep "Active" | awk '{print $3}')
    ACCEPTS=$(echo "$status_output" | grep "server accepts" | awk '{print $1}')
    HANDLED=$(echo "$status_output" | grep "server accepts" | awk '{print $2}')
    REQUESTS=$(echo "$status_output" | grep "server accepts" | awk '{print $3}')
    
    # Parse Reading/Writing/Waiting
    local rww_line=$(echo "$status_output" | grep "Reading")
    READING=$(echo "$rww_line" | awk '{print $2}')
    WRITING=$(echo "$rww_line" | awk '{print $4}')
    WAITING=$(echo "$rww_line" | awk '{print $6}')
    
    return 0
}

# Check backend health
check_backend_health() {
    local response_code
    response_code=$(curl -s -o /dev/null -w "%{http_code}" "$BACKEND_HEALTH_URL" 2>/dev/null)
    
    if [ "$response_code" = "200" ]; then
        return 0
    else
        log "WARNING: Backend health check failed (HTTP $response_code)"
        return 1
    fi
}

# Check for connection leaks
check_connection_leaks() {
    local backend_connections
    backend_connections=$(netstat -an | grep :9944 | wc -l)
    
    if [ "$backend_connections" -gt 100 ]; then
        log "WARNING: High backend connection count: $backend_connections"
        return 1
    fi
    
    return 0
}

# Send alert email
send_alert() {
    local subject="$1"
    local message="$2"
    
    if command -v mail > /dev/null 2>&1; then
        echo "$message" | mail -s "$subject" "$EMAIL_ALERT"
        log "INFO: Alert email sent: $subject"
    else
        log "WARNING: mail command not available, skipping email alert"
    fi
}

# Reload nginx with safety checks
safe_nginx_reload() {
    log "INFO: Performing nginx configuration test..."
    
    if nginx -t > /dev/null 2>&1; then
        log "INFO: Nginx configuration test passed, reloading..."
        nginx -s reload
        
        if [ $? -eq 0 ]; then
            log "SUCCESS: Nginx reloaded successfully"
            return 0
        else
            log "ERROR: Nginx reload failed"
            return 1
        fi
    else
        log "ERROR: Nginx configuration test failed, aborting reload"
        return 1
    fi
}

# Main monitoring function
monitor_connections() {
    if ! check_nginx_status; then
        return 1
    fi
    
    if ! get_connection_metrics; then
        return 1
    fi
    
    # Calculate connection efficiency
    local waiting_percent=0
    if [ "$ACTIVE_CONN" -gt 0 ]; then
        waiting_percent=$((WAITING * 100 / ACTIVE_CONN))
    fi
    
    # Color-coded output based on connection levels
    local color="$GREEN"
    local status="HEALTHY"
    
    if [ "$ACTIVE_CONN" -gt "$WARNING_THRESHOLD" ]; then
        color="$YELLOW"
        status="WARNING"
    fi
    
    if [ "$ACTIVE_CONN" -gt "$MAX_CONNECTIONS" ]; then
        color="$RED"
        status="CRITICAL"
    fi
    
    # Display current status
    printf "${color}[%s]${NC} Status: %s | Active: %d | Waiting: %d (%d%%) | R/W: %d/%d\n" \
           "$(date '+%H:%M:%S')" "$status" "$ACTIVE_CONN" "$WAITING" "$waiting_percent" "$READING" "$WRITING"
    
    # Log detailed metrics
    log "METRICS: Active=$ACTIVE_CONN, Waiting=$WAITING ($waiting_percent%), Reading=$READING, Writing=$WRITING"
    
    # Check for critical conditions
    if [ "$ACTIVE_CONN" -gt "$RELOAD_THRESHOLD" ]; then
        log "CRITICAL: Connection count exceeded reload threshold ($ACTIVE_CONN > $RELOAD_THRESHOLD)"
        
        send_alert "Dubhe WebSocket: Connection Overload" \
                  "Active connections: $ACTIVE_CONN\nWaiting connections: $WAITING\nAutomatic nginx reload initiated."
        
        if safe_nginx_reload; then
            log "INFO: Emergency nginx reload completed"
            sleep 10  # Give nginx time to settle
        else
            log "ERROR: Emergency nginx reload failed"
        fi
    elif [ "$waiting_percent" -gt 80 ] && [ "$ACTIVE_CONN" -gt 200 ]; then
        log "WARNING: High waiting connection percentage: $waiting_percent%"
        
        if ! check_backend_health; then
            log "WARNING: Backend health issues detected, considering reload"
        fi
    fi
    
    # Check for connection leaks
    if ! check_connection_leaks; then
        log "WARNING: Potential connection leaks detected"
    fi
    
    # Check backend health periodically
    if ! check_backend_health; then
        log "WARNING: Backend health check failed"
    fi
    
    return 0
}

# Cleanup function
cleanup() {
    log "INFO: Connection monitor shutting down..."
    exit 0
}

# Signal handlers
trap cleanup SIGINT SIGTERM

# Main execution
main() {
    log "INFO: Starting Dubhe WebSocket Connection Monitor"
    log "INFO: Configuration - Max: $MAX_CONNECTIONS, Reload: $RELOAD_THRESHOLD, Warning: $WARNING_THRESHOLD"
    
    # Create log directory if it doesn't exist
    mkdir -p "$(dirname "$LOG_FILE")"
    
    # Initial status check
    if ! monitor_connections; then
        log "ERROR: Initial status check failed"
        exit 1
    fi
    
    # Main monitoring loop
    while true; do
        monitor_connections
        sleep 30  # Monitor every 30 seconds
    done
}

# Show usage if help is requested
if [ "$1" = "--help" ] || [ "$1" = "-h" ]; then
    echo "Dubhe Channel WebSocket Connection Monitor"
    echo ""
    echo "Usage: $0 [options]"
    echo ""
    echo "Options:"
    echo "  --help, -h          Show this help message"
    echo "  --test              Run a single monitoring check and exit"
    echo "  --status            Show current connection status"
    echo ""
    echo "Configuration:"
    echo "  NGINX_STATUS_URL:   $NGINX_STATUS_URL"
    echo "  BACKEND_HEALTH_URL: $BACKEND_HEALTH_URL"
    echo "  MAX_CONNECTIONS:    $MAX_CONNECTIONS"
    echo "  RELOAD_THRESHOLD:   $RELOAD_THRESHOLD"
    echo "  WARNING_THRESHOLD:  $WARNING_THRESHOLD"
    echo "  LOG_FILE:           $LOG_FILE"
    echo ""
    exit 0
fi

# Test mode
if [ "$1" = "--test" ]; then
    log "INFO: Running single test check..."
    monitor_connections
    exit $?
fi

# Status mode
if [ "$1" = "--status" ]; then
    if get_connection_metrics; then
        echo "Current WebSocket Connection Status:"
        echo "  Active Connections: $ACTIVE_CONN"
        echo "  Waiting Connections: $WAITING"
        echo "  Reading: $READING"
        echo "  Writing: $WRITING"
        echo "  Total Accepts: $ACCEPTS"
        echo "  Total Handled: $HANDLED"
        echo "  Total Requests: $REQUESTS"
        
        if check_backend_health; then
            echo "  Backend Health: OK"
        else
            echo "  Backend Health: FAILED"
        fi
    else
        echo "ERROR: Failed to get connection metrics"
        exit 1
    fi
    exit 0
fi

# Run main function
main "$@" 