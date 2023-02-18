#!/usr/bin/env python

# Python modules
from dataclasses import dataclass
import sys
from typing import Iterable, Optional, List


@dataclass
class Bench(object):
    name: str
    inst: int
    l1: int
    l2: int
    ram: int
    cycles: int


def iter_blocks() -> Iterable[List[str]]:
    r: List[str] = []
    for line in sys.stdin:
        line = line.strip()
        if not line:
            yield r
            r = []
        else:
            r.append(line)
    if r:
        yield r


def is_valid(v: List[str]) -> bool:
    return len(v) == 6


def get_value(v: str) -> int:
    x = v.split(":", 1)[1].strip()
    if " " in x:
        return int(x.split()[0])
    return int(x)


def iter_benches() -> Iterable[Bench]:
    for block in iter_blocks():
        if not is_valid(block):
            continue
        yield Bench(
            name=block[0],
            inst=get_value(block[1]),
            l1=get_value(block[2]),
            l2=get_value(block[3]),
            ram=get_value(block[4]),
            cycles=get_value(block[5]),
        )


def main() -> None:
    benches = list(iter_benches())
    print(
        "| Name | Inst.[^1] | L1 Acc.[^2] | L2 Acc.[^3] | "
        "RAM Acc.[^4] | Est. Cycles [^5] |"
    )
    print("| --- | --: | --: | --: | --: | --: |")
    for b in benches:
        print(
            f"| {b.name} | {b.inst} | {b.l1} | {b.l2} | {b.ram} | {b.cycles} |"
        )


if __name__ == "__main__":
    main()
