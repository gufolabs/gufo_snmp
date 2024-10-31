# ---------------------------------------------------------------------
# Gufo SNMP: Utilities
# ---------------------------------------------------------------------
# Copyright (C) 2023-24, Gufo Labs
# See LICENSE.md for details
# ---------------------------------------------------------------------

"""Various utilities."""

# Python modules
from asyncio import Future, get_running_loop, wait_for
from asyncio import TimeoutError as AIOTimeoutError
from typing import Callable, Optional, TypeVar

# Gufo SNMP modules
from ..policer import BasePolicer

T = TypeVar("T")


async def send_and_recv(
    fd: int,
    sender: Callable[[], None],
    receiver: Callable[[], T],
    policer: Optional[BasePolicer],
    timeout: float,
) -> T:
    """
    Send request and receive response.

    Args:
        fd: File descriptor.
        sender: Callable which sends data.
        receiver: Callable which receives data.
        policer: Optional policer instance.
        timeout: Response timeout.
    """

    async def get_response() -> T:
        def read_callback() -> None:
            try:
                fut.set_result(receiver())
            except BaseException as e:  # noqa: BLE001
                fut.set_exception(e)

        while True:
            fut: Future[T] = loop.create_future()
            loop.add_reader(fd, read_callback)
            try:
                return await fut
            except BlockingIOError:
                continue
            finally:
                loop.remove_reader(fd)

    def write_callback() -> None:
        try:
            sender()
            fut.set_result(None)
        except BaseException as e:  # noqa: BLE001
            fut.set_exception(e)

    loop = get_running_loop()
    # Process limits
    if policer:
        await policer.wait()
    # Send request
    try:
        sender()
    except BlockingIOError:
        fut: Future[None] = loop.create_future()
        loop.add_writer(fd, write_callback)
        try:
            await fut
        finally:
            loop.remove_writer(fd)
    # Await response or timeout
    try:
        return await wait_for(get_response(), timeout)
    except AIOTimeoutError as e:
        raise TimeoutError from e  # Remap the error
