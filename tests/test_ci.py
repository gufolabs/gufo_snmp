# ---------------------------------------------------------------------
# Gufo Labs: CI test
# ---------------------------------------------------------------------
# Copyright (C) 2022-23, Gufo Labs
# See LICENSE.md for details
# ---------------------------------------------------------------------

# Python modules
import inspect
import os
import sys
from typing import Iterable, Tuple

# Third-party modules
import pytest
import yaml


def _get_root() -> str:
    mod_path = inspect.getfile(sys.modules[__name__])
    rel_root = os.path.dirname(mod_path)
    return os.path.abspath(os.path.join(rel_root, ".."))


VERSIONS = [
    "actions/cache@v3",
    "actions/checkout@v3",
    "actions/setup-python@v4",
    "pypa/gh-action-pypi-publish@release/v1",
]


def _iter_actions() -> Iterable[Tuple[str, str, str, str, str]]:
    versions = {a.split("@")[0]: a.split("@")[1] for a in VERSIONS}
    root = os.path.join(_get_root(), ".github", "workflows")
    for f in os.listdir(root):
        if f.startswith(".") or not f.endswith(".yml"):
            continue
        path = os.path.join(root, f)
        with open(path) as f:
            data = yaml.safe_load(f)
        for job in data["jobs"]:
            for step in data["jobs"][job]["steps"]:
                if "uses" in step:
                    uses = step["uses"]
                    for v in versions:
                        if uses.startswith(f"{v}@"):
                            yield path, job, step["name"], v, uses.split("@")[
                                1
                            ], versions[v]
                            break


@pytest.mark.parametrize(
    ("path", "job", "step", "action", "ver", "exp"), list(_iter_actions())
)
def test_actions(
    path: str, job: str, step: str, action: str, ver: str, exp: str
) -> None:
    assert (
        ver == exp
    ), f"{path}:{job}/{step}: {action}@{exp} required (@{ver} used)"
