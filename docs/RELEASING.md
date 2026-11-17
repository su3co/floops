# Python releases

The initial distribution is **floops** on PyPI, using a native Rust backend. Crates.io publishing is disabled in every Cargo package.

## One-time PyPI setup

Add a pending GitHub trusted publisher at <https://pypi.org/manage/account/publishing/>:

| Field | Value |
| --- | --- |
| PyPI project name | `floops` |
| Owner | `su3co` |
| Repository | `floops` |
| Workflow | `release.yml` |
| Environment | `pypi` |

The workflow lives at `.github/workflows/release.yml`. A pending publisher does not reserve the project name; the first successful upload creates the project. Publishing uses GitHub's short-lived OIDC identity, not a stored PyPI token.

## Build and publish

1. Keep the workspace version, `pyproject.toml`, local dependency versions and smoke-test version in sync.
2. Run the **Release Python package** workflow with `publish=false` to prepare and test distributions.
3. After configuring the trusted publisher, run it with `publish=true`, or push a matching `vVERSION` tag.
4. Check the workflow's publish job and the resulting PyPI project, then install the published wheel in a fresh environment and run `examples/python_smoke.py`.

The workflow builds ABI-stable CPython 3.10+ wheels for Linux x86_64/ARM64, macOS Intel/Apple Silicon and Windows x86_64. Each wheel is tested on Python 3.10 and 3.15. The source archive is independently installed and exercised. Linux wheels target manylinux2014. Python-free-threaded and PyPy builds are not part of this release matrix.

Publishing is allowed only after all wheel and source checks pass. PyPI does not allow replacing an uploaded filename; correct release errors with a new version.
