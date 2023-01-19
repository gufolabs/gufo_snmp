# ---------------------------------------------------------------------
# Gufo Labs: CI test
# ---------------------------------------------------------------------
# Copyright (C) 2022, Gufo Labs
# See LICENSE.md for details
# ---------------------------------------------------------------------

# Python modules
import os

# Third-party modules
import yaml
import pytest
import sys
import inspect


def _get_root():
    mod_path = inspect.getfile(sys.modules[__name__])
    rel_root = os.path.dirname(mod_path)
    return os.path.abspath(os.path.join(rel_root, ".."))


VERSIONS = [
    "actions/cache@v3",
    "actions/checkout@v3",
    "actions/setup-python@v4",
    "pypa/gh-action-pypi-publish@release/v1",
]


def _iter_actions():
    versions = {a.split("@")[0]: a.split("@")[1] for a in VERSIONS}
    root = os.path.join(_get_root(), ".github", "workflows")
    for f in os.listdir(root):
        if f.startswith(".") or not f.endswith(".yml"):
            continue
        path = os.path.join(root, f)
        with open(path) as f:
            data = yaml.load(f, yaml.Loader)
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


@pytest.mark.parametrize("path,job,step,action,ver,exp", list(_iter_actions()))
def test_actions(path, job, step, action, ver, exp):
    assert (
        ver == exp
    ), f"{path}:{job}/{step}: {action}@{exp} required (@{ver} used)"
