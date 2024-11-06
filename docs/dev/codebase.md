# Project's Code Base

The code base of the project has following structure:

* `.devcontainer/` - Developer's container configuration for 
  [VSCode Remote Containers][Remote Containers]. Just reopen
  project in remote container to get ready-to-development
  environment.
* `.github/` - GitHub settings

    * `workflows/` - [GitHub Actions Workflows][GitHub Workflows] settings.
      Used to run tests and build the documentation.

* `.requirements/` - Python dependencies for development environment.

    * `build.txt` - Setuptools build requirements.
    * `docs.txt` - [Mkdocs Material][Mkdocs Material] dependencies.
    * `ipython.txt` - [IPython] dependencies.
    * `lint.txt` - [Ruff][Ruff] and [Mypy][Mypy] dependencies.
    * `test.txt` - [Pytest][Pytest] dependencies.

* `docs/` - [Mkdocs][Mkdocs] documentation.
* `examples/` - Project's examples.
* `src/` - Project's source code.
* `tests/` - Project's [Pytest][Pytest] test suite.
* `.gitignore` - [Gitignore][Gitignore] file.
* `Dockerfile` - [Dockerfile][Dockerfile] for development container.
* `mkdocs.yml` - [Mkdocs][Mkdocs] configuration file.
* `pyproject.toml` - [pyproject.toml][Pyproject] file for python tools configuration.

[Remote Containers]: https://marketplace.visualstudio.com/items?itemName=ms-vscode-remote.remote-containers
[GitHub Workflows]: https://docs.github.com/en/actions/using-workflows
[Mkdocs]: https://www.mkdocs.org
[Mkdocs Material]: https://squidfunk.github.io/mkdocs-material/
[Ruff]: https://docs.astral.sh/ruff/
[Mypy]: https://mypy.readthedocs.io/en/stable/
[Pytest]: https://docs.pytest.org/
[Dockerfile]: https://docs.docker.com/engine/reference/builder/
[Gitignore]: https://git-scm.com/docs/gitignore
[Pyproject]: https://pip.pypa.io/en/stable/reference/build-system/pyproject-toml/
