# MOSS Decoder
[![CI](https://github.com/CramBL/moss_decoder/actions/workflows/CI.yml/badge.svg)](https://github.com/CramBL/moss_decoder/actions/workflows/CI.yml)
![PyPI - Downloads](https://img.shields.io/pypi/dm/moss-decoder)
![GitHub commit activity (branch)](https://img.shields.io/github/commit-activity/m/crambl/moss_decoder)


Python module implemented in Rust for decoding raw data from the MOSS chip (Stitched Monolithic Pixel Sensor prototype).
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
