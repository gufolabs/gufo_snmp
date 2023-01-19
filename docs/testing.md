# Building and Testing

Before starting building and testing package set up 
[Developer's Environment](environment.md) first.
From here and below we consider the shell's current
directory matches the project's root directory.

## Building Package

To test the package build run:

```
$ python -m build --sdist --wheel
```

Compiled packages will be available in the `dist/` directory.

## Rebuilding Rust Modules

To rebuild Rust modules for debug and testing:

```
$ python setup.py build_rust
```

## Running tests

Rebuild rust modules, if changed:

```
$ python setup.py develop
```

To run the test suit:

```
$ pytest -vv
```

## Running Lints

All lints are checked as part of GitHub Actions Workflow. You may run lints
manually before committing to the project.

### Check Formatting

[Python Code Formatting](codequality.md#python-code-formatting) is the mandatory
requirement in our [Code Quality](codequality.md) standards. To check code
formatting run:

```
$ black --check examples/ src/ tests/
```

To fix formatting errors run:
```
$ black examples/ src/ tests/
```

We recommend setting python code formatting on file saving
(Done in [VS Code Dev Container](environment.md#visual-studio-code-dev-container)
out of the box).

### Python Code Lints

[Python Code Linting](codequality.md#python-code-linting) is the mandatory
requirement in our [Code Quality](codequality.md) standards. To check code
for linting errors run:

```
$ flake8 examples/ src/ tests/
```

### Python Code Static Checks

[Python Code Static Checks](codequality.md#python-code-static-checks) is the mandatory
requirement in our [Code Quality](codequality.md) standards. To check code
for typing errors run:

```
$ mypy --strict src/
```

## Python Test Code Coverage Check

To evaluate code coverage run tests:

```
$ coverage run -m pytest -vv
```

To report the coverage after the test run:

```
$ coverage report
```

To show line-by-line coverage:

```
$ coverage html
```

Then open `dist/coverage/index.html` file in your browser.

## Building Documentation

To rebuild and check documentation run

```
$ mkdocs serve
```

We recommend using [Grammarly][Grammarly] service to check
documentation for common errors.

[Grammarly]: https://grammarly.com/