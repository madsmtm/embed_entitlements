#!/bin/sh

# Super simple script for spawning binary on the simulator.
#
# For more real-world use-cases where you want to bundle as well, see:
# https://simlay.net/posts/rust-target-runner-for-ios/

set -euo pipefail

EXECUTABLE=$1
ARGS=${@:2}

# Overly simplified, real-world use-cases should find the relevant
# device depending on binary instead of just using `booted`.
DEVICE=booted

# Copy executable to temporary location on device.
#
# This is done to make the executable readable so that the
# `binary_inspection.rs` test works.
DEVICE_TMPDIR=$(xcrun simctl getenv $DEVICE TMPDIR)
DEVICE_EXECUTABLE=$(mktemp $DEVICE_TMPDIR/$(basename $EXECUTABLE).XXXXXX)
cp -c $EXECUTABLE $DEVICE_EXECUTABLE

# Spawn the executable with the arguments.
xcrun simctl spawn $DEVICE $DEVICE_EXECUTABLE $ARGS

# Remove file again.
rm $DEVICE_EXECUTABLE
