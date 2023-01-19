# Code Quality Guide

We share the common code quality standards between all Gufo Labs projects.

## Python Code Formatting

All Python code must be formatting using [Black][Black] code formatter
with settings defined in the project's `pyproject.toml` file.

## Python Code Linting

* All Python code must satisfy [PEP8][PEP8] standards.
* Code must not contain unused imports.
* Code must not contain unused variables.
* Code must not use `l` variable or function names.

All python code must pass [Flake8][Flake8] tests.

## Python Code Static Checks

All python code must pass [Mypy][Mypy] type checks in the `strict` mode.

## Test Suite Coverage

The test suite must provide 100% code coverage whenever possible.

## Documentation Standards

* Documentation must be clean and mean.

[Black]: https://black.readthedocs.io/en/stable
[Flake8]: https://flake8.pycqa.org/en/latest/
[Mypy]: https://mypy.readthedocs.io/en/stable/
[PEP8]: https://peps.python.org/pep-0008/