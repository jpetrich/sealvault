#!/usr/bin/env python3
"""Run builds, linters and tests.
"""

import argparse
from pathlib import Path
import subprocess as sp
import signal


DEV_SERVER_PROCESS = None
REPO_ROOT = Path(__file__).parent
DEV_SERVER_DIR = REPO_ROOT / "tools/dev-server"
IOS_DIR = REPO_ROOT / "ios"


def main():
    static_analysis()

    run_rust_tests()

    start_dev_server()
    run_ios_ui_tests()


def static_analysis():
    mpl_header_check()
    run_rustfmt_check()
    run_clippy()
    # Platform specific code only gets linted when compiled for that target.
    run_clippy("aarch64-apple-ios")


def cleanup():
    if DEV_SERVER_PROCESS is not None:
        print("Killing dev server process")
        DEV_SERVER_PROCESS.kill()


def signal_handler(signum, frame):
    print("Signal handler called with signal", signum)
    cleanup()


def start_dev_server():
    global DEV_SERVER_PROCESS

    DEV_SERVER_PROCESS = sp.Popen(["cargo", "run"], cwd=DEV_SERVER_DIR)


def mpl_header_check():
    pattern = """
^// This Source Code Form is subject to the terms of the Mozilla Public$
^// License, v. 2.0. If a copy of the MPL was not distributed with this$
^// file, You can obtain one at https://mozilla.org/MPL/2.0/.$
    """.strip()
    # Current directory at end is important otherwise fails in GitHub CI
    command = f'rg --files-without-match --multiline -trust -tswift -tjs "{pattern}" ./'
    result = sp.run(command, capture_output=True, shell=True, text=True)
    # 0 means match and 1 means no match which is what we want
    if result.returncode > 1:
        print("rg returned failed with error", result.returncode)
        print(result.stderr)
        exit(result.returncode)

    if result.stdout:
        print("Linter error. All source files must start with the MPL header:")
        regex_chars = dict.fromkeys(map(ord, '^$'), None)
        print(pattern.translate(regex_chars))
        print("The following files are missing the header:")
        print(result.stdout)
        exit(1)


def run_clippy(target=None):
    target_arg = f"--target {target} " if target else ""
    # The `uninlined-format-args` rule is disabled, because code generated by uniffi doesn't comply with it + not all
    # format args can be inlined in which case it's better to not inline any of them.
    allow = "-A clippy::uninlined-format-args"

    sp.run(
        f"cargo clippy {target_arg}-- -D warnings {allow}".split(" "),
        check=True,
    )


def run_rustfmt_check():
    sp.run(
        f"cargo fmt --check --all".split(" "),
        check=True,
    )


def run_rust_tests():
    sp.run(["cargo", "test", "--package", "sealvault_core"], check=True)


def run_ios_ui_tests():
    sp.run(["fastlane", "tests"], check=True, cwd=IOS_DIR)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--static",
        action="store_true",
        help="Run static analysis only",
    )
    args = parser.parse_args()

    # Finally block is not enough in case another interrupt is received
    # while it's executing.
    signal.signal(signal.SIGINT, signal_handler)

    try:
        if args.static:
            static_analysis()
        else:
            main()
    finally:
        cleanup()

    print("Success")
