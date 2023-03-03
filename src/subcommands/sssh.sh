#!/usr/bin/env sh

# `sssh` runs this script with the following arguments:
#    /path/to/this/script "<USERNAME>@<HOSTNAME>" "<USERNAME>" "<HOSTNAME>" "<PORT>" "<DEBUG>"
# It also sets the following environment variables:
#    SSSH_ADDRESS  = "<USERNAME>@<HOSTNAME>"
#    SSSH_USERNAME = "<USERNAME>"
#    SSSH_HOSTNAME = "<HOSTNAME>"
#    SSSH_PORT     = "<PORT>"
#    SSSH_DEBUG    = "<DEBUG>"
# If `sssh` itself is started with --verbose (or -v), <DEBUG> will be "1", otherwise "0"

set -e
if [ "${SSSH_DEBUG}" = "1" ]; then
  # `sssh` forwards its own logging messages to `stderr`.
  # We recommend you to do the same here.
  echo "Arguments:     " "$@"               >&2
  echo "SSSH_ADDRESS:  " "${SSSH_ADDRESS}"  >&2
  echo "SSSH_USERNAME: " "${SSSH_USERNAME}" >&2
  echo "SSSH_HOSTNAME: " "${SSSH_HOSTNAME}" >&2
  echo "SSSH_PORT:     " "${SSSH_PORT}"     >&2
  set -xe
fi

ssh -p "${SSSH_PORT}" "${SSSH_ADDRESS}"
