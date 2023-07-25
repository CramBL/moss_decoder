# MOSS Decoder
[![CI](https://github.com/CramBL/moss_decoder/actions/workflows/CI.yml/badge.svg)](https://github.com/CramBL/moss_decoder/actions/workflows/CI.yml)
![PyPI - Downloads](https://img.shields.io/pypi/dm/moss-decoder)
![GitHub commit activity (branch)](https://img.shields.io/github/commit-activity/m/crambl/moss_decoder)


Python module implemented in Rust for decoding raw data from the MOSS chip (Stitched Monolithic Pixel Sensor prototype).

- [MOSS Decoder](#moss-decoder)
  - [Installation](#installation)
  - [Motivation \& Purpose](#motivation--purpose)
  - [@CERN Gitlab installation for CentOS and similar distributions from local build](#cern-gitlab-installation-for-centos-and-similar-distributions-from-local-build)
    - [Troubleshooting](#troubleshooting)

## Installation
```shell
$ pip install moss-decoder
```
Import in python and use for decoding raw data.
```python
import moss_decoder
moss_packets = moss_decoder.decode_event(bytes)
```
## Motivation & Purpose
Decoding in native Python is slow and the MOSS verification team at CERN got to a point where we needed more performance.

Earliest version of a Rust package gave massive improvements as shown in the benchmark below.

Decoding the 10 MB MOSS readout data. Performed on CentOS Stream 9 with Python 3.11

| Command                                          |       Mean [s] | Min [s] | Max [s] |      Relative |
| :----------------------------------------------- | -------------: | ------: | ------: | ------------: |
| `python moss_test/util/decoder_native_python.py` | 36.319 ± 0.175 |  36.057 |  36.568 | 228.19 ± 2.70 |
| `python moss_test/util/decoder_rust_package.py`  |  0.159 ± 0.002 |   0.157 |   0.165 |          1.00 |

## @CERN Gitlab installation for CentOS and similar distributions from local build

If you update the package source code and want to build and install without publishing and fetching from PyPI, you can follow these steps.

The `.gitlab-ci.yml` file contains a `build-centos` manual job which will build the MOSS decoder package from source and saves the package as an artifact.

1. Start the job, download the artifacts.
2. Unzip the artifacts and you will find a `wheels` package in `/target/wheels/` with the `.whl` extension
3. Run `python -m pip install <wheels package>.whl`
4. Confirm the installation with `python -m pip freeze | grep moss`, it should display something containing `moss_decoder @ file:<your-path-to-wheels-package>`

### Troubleshooting
if you get `ERROR: Could not find a version that satisfies the requirement ...` make sure to add `.whl` when performing step 3 above.

if you don't see the expected message at step 4, try running the installation command in step 3 with `--upgrade` and/or `--no-cache-dir`.
