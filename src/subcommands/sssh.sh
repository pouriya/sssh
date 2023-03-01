#!/usr/bin/env sh

set -e
if [ "${DEBUG}" = "1" ]; then
  echo "Arguments:" "$@"
  echo "ADDRESS:"   "${ADDRESS}"
  echo "USERNAME:"  "${USERNAME}"
  echo "HOSTNAME:"  "${HOSTNAME}"
  echo "PORT:"      "${PORT}"
  set -xe
fi

ssh -p "${PORT}" "${USERNAME}@${HOSTNAME}"
