#!/bin/sh
# ------------------------------------------------------------------------
# Gufo Labs: Install snmpd
# ------------------------------------------------------------------------
# Copyright (C) 2023, Gufo Labs
# ------------------------------------------------------------------------

set -e
OS="unknown"

if  [ -f /etc/redhat-release ]; then
    OS="rhel"
elif [ -f /etc/debian_version ]; then
    OS="debian"
elif [ -f /etc/alpine-release ]; then
    OS="alpine"
elif [ "$OSNAME" == "Darwin" ]; then
    OS="darwin"    
else
    echo "Cannot detect OS"
    exit 1
fi

if [ $(id -u) -eq 0 ]; then
    SUDO=""
else
    SUDO="sudo"
fi

echo "Installing snmpd for $OS"
case $OS in
    rhel)
        $SUDO yum install -y net-snmp
        # Test
        /usr/sbin/snmpd --version
        ;;
    debian)
        $SUDO apt-get update
        $SUDO apt-get install -y --no-install-recommends snmpd
        # Test
        /usr/sbin/snmpd --version
        ;;
    alpine)
        $SUDO apk add net-snmp
        ;;
    darwin)
        brew install net-snmp
        ;;        
    *)
        echo "Unsupported OS: $OS"
        exit 1
        ;;
esac