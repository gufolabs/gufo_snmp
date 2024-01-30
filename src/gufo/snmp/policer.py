# ---------------------------------------------------------------------
# Gufo SNMP: Query policers
# ---------------------------------------------------------------------
# Copyright (C) 2023, Gufo Labs
# See LICENSE.md for details
# ---------------------------------------------------------------------

"""Policer abstract base class and the implementations."""

# Python modules
import asyncio
from abc import ABC, abstractmethod
from time import perf_counter_ns, sleep
from typing import Optional

NS = 1_000_000_000.0
ZERO = 0.0


class BasePolicer(ABC):
    """
    Base query policer.

    Query policer limits the rate of
    outgoing GET/GETNEXT/GETBULK queries
    up to the desired rate.

    This is abstract class, the real
    implemetation must be defined in subclasses.

    The instances of the subclasses can be used
    as SnmpSession.policer parameters.
    """

    @abstractmethod
    def get_timeout(self: "BasePolicer", ts: int) -> Optional[int]:
        """
        Get sleep timeout.

        Calculate next sleep timeout and adjust the internal state.
        Separate function for the testing purposes.

        Args:
            ts: Result of perf_counter_ns()

        Returns:
            * None, if the call is not policed.
            * sleep timeout in nanoseconds, otherwise.
        """

    async def wait(self: "BasePolicer") -> None:
        """
        Apply policy.

        Waits until the sending of the next request
        will be possible according to the policy.

        May be interrupted by TimeoutError.
        """
        delta = self.get_timeout(perf_counter_ns())
        if delta and delta > 0:
            await asyncio.sleep(float(delta) / NS)

    def wait_sync(self: "BasePolicer") -> None:
        """
        Apply policy  (Synchronous version).

        Waits until the sending of the next request
        will be possible according to the policy.

        May be interrupted by TimeoutError.
        """
        delta = self.get_timeout(perf_counter_ns())
        if delta and delta > 0:
            sleep(float(delta) / NS)


class RPSPolicer(BasePolicer):
    """
    Requests per seconds policer.

    Aligns queries up to `rps` requests
    per seconds and tries to arrange queries
    to the equal intervals.

    Args:
        rps: Requests per second rate.
    """

    def __init__(self: "RPSPolicer", rps: float) -> None:
        if rps <= ZERO:
            msg = "Invalid RPS"
            raise ValueError(msg)
        self._prev: Optional[int] = None
        self._delta: int = int(NS / rps)
        if not self._delta:
            msg = "RPS is too high"
            raise ValueError(msg)

    def get_timeout(self: "RPSPolicer", ts: int) -> Optional[int]:
        """
        Get sleep timeout.

        Calculate next sleep timeout and adjust the internal state.
        Separate function for the testing purposes.

        Args:
            ts: Result of perf_counter_ns()

        Returns:
            * None, if the call is not policed.
            * sleep timeout in nanoseconds, otherwise.
        """
        if self._prev is None:
            # First call, pass immediately
            self._prev = ts
            return None
        elapsed = ts - self._prev
        if elapsed < 0:
            # CPU timer stepback,
            # Possible VM migration.
            # Start from scratch
            self._prev = ts
            return self._delta
        if elapsed < self._delta:
            # Not enough time passed,
            # have to sleep
            self._prev += self._delta
            return self._delta - elapsed
        # One or more interval passed
        self._prev += self._delta * (elapsed // self._delta)
        return None
