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
else
    echo "Cannot detect OS"
    exit 1
fi

echo "Installing snmpd for $OS"
case $OS in
    rhel)
        yum install -y net-snmp
        ;;
    debian)
        apt-get update
        apt-get install -y --no-install-recommends snmpd
        ;;
    alpine)
        apk add net-snmp
        ;;
    *)
        echo "Unsupported OS: $OS"
        exit 1
        ;;
esac