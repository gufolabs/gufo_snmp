# ----------------------------------------------------------------------
# Gufo Traceroute: docs tests
# ----------------------------------------------------------------------
# Copyright (C) 2022-23, Gufo Labs
# See LICENSE.md for details
# ----------------------------------------------------------------------

# Python modules
import os
import re
from functools import lru_cache
from typing import List, Optional, Set

# Third-party modules
import pytest

_doc_files: Optional[List[str]] = None

rx_link = re.compile(r"\[([^\]]+)\]\[([^\]]+)\]", re.MULTILINE)
rx_link_def = re.compile(r"^\[([^\]]+)\]:", re.MULTILINE)
rx_footnote = re.compile(r"[^\]]\[(\^\d+)\][^\[]", re.MULTILINE)


@lru_cache(maxsize=1)
def get_docs() -> List[str]:
    doc_files: List[str] = []
    for root, _, files in os.walk("docs"):
        for f in files:
            if f.endswith(".md") and not f.startswith("."):
                doc_files.append(os.path.join(root, f))
    return doc_files


def get_file(path: str) -> str:
    with open(path) as f:
        return f.read()


@pytest.mark.parametrize("doc", get_docs())
def test_links(doc: str) -> None:
    data = get_file(doc)
    links: Set[str] = set()
    defs: Set[str] = set()
    for match in rx_link.finditer(data):
        links.add(match.group(2))
    for match in rx_footnote.finditer(data):
        print(match.group(1))
        links.add(match.group(1))
    for match in rx_link_def.finditer(data):
        d = match.group(1)
        assert d not in defs, f"Link already defined: {d}"
        assert d in links, f"Unused link definition: {d}"
