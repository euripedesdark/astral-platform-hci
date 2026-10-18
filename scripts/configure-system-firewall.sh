#!/usr/bin/env bash
set -Eeuo pipefail
[[ $EUID -eq 0 ]]||{ echo "Execute com sudo.";exit 1; }
if command -v firewall-cmd >/dev/null && firewall-cmd --state >/dev/null 2>&1; then
 firewall-cmd --permanent --add-port=8080/tcp >/dev/null
 firewall-cmd --reload >/dev/null
else
 iptables -C INPUT -p tcp --dport 8080 -j ACCEPT 2>/dev/null||iptables -I INPUT 1 -p tcp --dport 8080 -j ACCEPT
 if command -v netfilter-persistent >/dev/null;then netfilter-persistent save >/dev/null 2>&1||true
 elif command -v iptables-save >/dev/null;then mkdir -p /etc/iptables;iptables-save >/etc/iptables/rules.v4.tmp && mv /etc/iptables/rules.v4.tmp /etc/iptables/rules.v4
 fi
fi
