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
from dataclasses import dataclass
from typing import Iterable

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


@dataclass
class Action(object):
    path: str
    job: str
    step: str
    action: str
    version: str
    expected: str


def action_label(action: Action) -> str:
    return f"{action.path}: {action.job}/{action.step}: {action.action}"


def _iter_actions() -> Iterable[Action]:
    versions = {a.split("@")[0]: a.split("@")[1] for a in VERSIONS}
    root = os.path.join(_get_root(), ".github", "workflows")
    for fn in os.listdir(root):
        if fn.startswith(".") or not fn.endswith(".yml"):
            continue
        path = os.path.join(root, fn)
        with open(path) as f:
            data = yaml.safe_load(f)
        for job in data["jobs"]:
            for step in data["jobs"][job]["steps"]:
                if "uses" in step:
                    uses = step["uses"]
                    for v in versions:
                        if uses.startswith(f"{v}@"):
                            yield Action(
                                path=fn,
                                job=job,
                                step=step["name"],
                                action=v,
                                version=uses.split("@")[1],
                                expected=versions[v],
                            )
                            break


@pytest.mark.parametrize("action", list(_iter_actions()), ids=action_label)
def test_actions(action: Action) -> None:
    loc = f"{action.path}: {action.job}/{action.step}"
    v_exp = f"{action.action}@{action.expected}"
    msg = f"{loc}: {v_exp} required (@{action.version} used)"
    assert action.version == action.expected, msg
