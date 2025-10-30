# Bash script for setting up the environmental variables.
# Before running this, install vcpkg and the required packages (see README.md).
# To setup the environment, source this file.
export VCPKG_ROOT=$(realpath "../vcpkg/")
export RUSTFLAGS='-Ctarget-feature=+crt-static'
export VCPKGRS_TRIPLET='x64-windows-static-release'
export TECTONIC_DEP_BACKEND='vcpkg'
