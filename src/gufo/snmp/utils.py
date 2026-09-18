# ---------------------------------------------------------------------
# Gufo SNMP: Various utilities
# ---------------------------------------------------------------------
# Copyright (C) 2023-26, Gufo Labs
# See LICENSE.md for details
# ---------------------------------------------------------------------

"""Various utilities."""


def format_sock_addr(addr: str, port: int) -> str:
    """Format an address and port as a socket address.

    Args:
        addr: IP address or hostname.
        port: Port number.

    Returns:
        Address formatted as ``host:port``,
        with IPv6 addresses enclosed in square brackets.
    """
    if ":" in addr and not addr.startswith("[") and not addr.endswith("]"):
        return f"[{addr}]:{port}"  # IPv6, add brackets manually
    return f"{addr}:{port}"
