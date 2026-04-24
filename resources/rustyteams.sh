#!/bin/sh
# RustyTeams launcher.
#
# The Debian package installs the binary, libcef.so, and all CEF runtime
# resources (.pak / icudtl.dat / locales / snapshots) into /opt/rustyteams/.
# CEF resolves those resources relative to the executable's own directory,
# so we exec from /opt/rustyteams/ and extend LD_LIBRARY_PATH to pick up
# libcef.so and friends without requiring an rpath in the binary.

APP_DIR=/opt/rustyteams
LD_LIBRARY_PATH="${APP_DIR}${LD_LIBRARY_PATH:+:${LD_LIBRARY_PATH}}" \
    exec "${APP_DIR}/rustyteams" "$@"
