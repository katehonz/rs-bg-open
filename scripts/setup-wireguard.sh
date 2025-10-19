#!/bin/bash

################################################################################
# WireGuard Setup Script
#
# Този скрипт конфигурира WireGuard tunnel за IPv6 connectivity
#
# Usage: ./setup-wireguard.sh
################################################################################

set -e

GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}  WireGuard IPv6 Setup${NC}"
echo -e "${GREEN}========================================${NC}"
echo ""

# Check if running with sudo
if [[ $EUID -ne 0 ]] && ! groups | grep -q docker; then
   echo -e "${RED}Error: This script needs to be run with docker permissions${NC}"
   echo -e "${YELLOW}Either run with sudo or add your user to docker group:${NC}"
   echo -e "  sudo usermod -aG docker \$USER"
   exit 1
fi

# Check if WireGuard config exists
if [ ! -f "IPV6/wireguard/wg0.conf" ]; then
    echo -e "${YELLOW}WireGuard config not found!${NC}"
    echo ""
    echo "Please create IPV6/wireguard/wg0.conf with your configuration from ipv6.rs"
    echo ""
    echo "Example:"
    echo "  [Interface]"
    echo "  PrivateKey = YOUR_PRIVATE_KEY"
    echo "  Address = YOUR_IPV6_ADDRESS/64"
    echo "  "
    echo "  [Peer]"
    echo "  PublicKey = SERVER_PUBLIC_KEY"
    echo "  Endpoint = SERVER_ENDPOINT:51820"
    echo "  AllowedIPs = ::/0"
    echo "  PersistentKeepalive = 25"
    echo ""
    exit 1
fi

# Check if SSL certificates exist
if [ ! -f "IPV6/ssl/certificate.crt" ] || [ ! -f "IPV6/ssl/private.key" ]; then
    echo -e "${YELLOW}SSL certificates not found!${NC}"
    echo ""
    echo "Please add your ZeroSSL certificates to IPV6/ssl/:"
    echo "  - certificate.crt"
    echo "  - ca_bundle.crt"
    echo "  - private.key"
    echo ""
    exit 1
fi

# Set proper permissions
echo -e "${YELLOW}Setting file permissions...${NC}"
chmod 600 IPV6/wireguard/wg0.conf
chmod 644 IPV6/ssl/certificate.crt
chmod 600 IPV6/ssl/private.key
if [ -f "IPV6/ssl/ca_bundle.crt" ]; then
    chmod 644 IPV6/ssl/ca_bundle.crt
fi

# Enable IPv6 on the host
echo -e "${YELLOW}Enabling IPv6 on host...${NC}"
if [[ $EUID -eq 0 ]]; then
    sysctl -w net.ipv6.conf.all.disable_ipv6=0
    sysctl -w net.ipv6.conf.default.disable_ipv6=0
    sysctl -w net.ipv6.conf.all.forwarding=1
else
    echo -e "${YELLOW}Skipping sysctl (requires sudo)${NC}"
fi

# Enable IPv6 in Docker daemon
echo -e "${YELLOW}Checking Docker IPv6 configuration...${NC}"
if [ -f "/etc/docker/daemon.json" ]; then
    if ! grep -q "ipv6" /etc/docker/daemon.json; then
        echo -e "${YELLOW}IPv6 not enabled in Docker daemon${NC}"
        echo -e "${YELLOW}Add this to /etc/docker/daemon.json:${NC}"
        echo '{'
        echo '  "ipv6": true,'
        echo '  "fixed-cidr-v6": "fd00::/80"'
        echo '}'
        echo ""
        echo -e "${YELLOW}Then restart Docker: sudo systemctl restart docker${NC}"
    fi
fi

echo ""
echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}  Setup Complete!${NC}"
echo -e "${GREEN}========================================${NC}"
echo ""
echo -e "${YELLOW}Next steps:${NC}"
echo ""
echo "1. Make sure your .env file is configured:"
echo "   cp .env.production.example .env"
echo "   # Edit .env with your settings"
echo ""
echo "2. Start the services:"
echo "   docker-compose up -d"
echo ""
echo "3. Check WireGuard status:"
echo "   docker exec wireguard wg show"
echo ""
echo "4. Test IPv6 connectivity:"
echo "   docker exec wireguard ping6 -c 3 google.com"
echo ""
echo "5. Access your app at:"
echo "   https://rs.ddns-ip.net"
echo ""
