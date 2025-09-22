# ---------------------------------------------------------------------
# Gufo Labs: Test Gufo SNMP
# ---------------------------------------------------------------------
# Copyright (C) 2023, Gufo Labs
# See LICENSE.md for details
# ---------------------------------------------------------------------

# Python modules
import asyncio
from time import perf_counter_ns

# Third-party modules
import pytest

# Gufo Labs modules
from gufo.snmp.policer import BasePolicer, RPSPolicer


def test_base_instance() -> None:
    with pytest.raises(TypeError):
        BasePolicer()


@pytest.mark.parametrize("rps", [0.0, -1.0, 10_000_000_000.0])
def test_invalid_rps(rps: float) -> None:
    with pytest.raises(ValueError):
        RPSPolicer(rps)


def test_rps_timeout() -> None:
    def T(s: int, t: int = 0) -> int:
        return t0 + s * step + t * tick

    t0 = 0  # perf_counter_ns()
    rps = 10
    step = 1_000_000_000 // rps
    tick = step // 4

    # ts, prev, timeout
    scenario = [
        (T(0), T(0), None),
        (T(0, 3), T(1), tick),
        (T(1, 1), T(2), 3 * tick),
        (T(2), T(3), 4 * tick),
        (T(4), T(4), None),
        (T(7, 1), T(7), None),
    ]
    p = RPSPolicer(rps)
    for ts, prev, timeout in scenario:
        print(f"#  ts={ts:,} prev={p._prev}")
        r = p.get_timeout(ts)
        assert timeout == r
        assert p._prev == prev


def test_rps_flood_async() -> None:
    async def inner() -> None:
        p = RPSPolicer(rps)
        # Emulate flood of requests
        for _ in range(requests):
            await p.wait()

    rps = 10
    duration = 2
    requests = duration * rps + 1
    t0 = perf_counter_ns()
    asyncio.run(inner())
    delta = perf_counter_ns() - t0
    # Check duration
    assert delta >= duration * 1_000_000_000


def test_rps_flood_sync() -> None:
    rps = 10
    duration = 2
    requests = duration * rps + 1
    t0 = perf_counter_ns()
    p = RPSPolicer(rps)
    # Emulate flood of requests
    for _ in range(requests):
        p.wait_sync()
    delta = perf_counter_ns() - t0
    # Check duration
    assert delta >= duration * 1_000_000_000
