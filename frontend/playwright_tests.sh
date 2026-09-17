#!/usr/bin/env bash

# ==============================================================================

# Fail fast logic: 
# -E: ERR trap is inherited by shell functions.
# -e: Exit immediately if a command exits with a non-zero status.
# -u: Treat unset variables as an error.
# -o pipefail: Pipeline status is the value of the last command to exit with a non-zero status.
set -Eeuo pipefail

# Track the script's native directory regardless of where it is run from
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" &>/dev/null && pwd -P)
SCRIPT_NAME=$(basename "${BASH_SOURCE[0]}")
cd "${SCRIPT_DIR}" || exit 1

# ------------------------------------------------------------------------------
# Default Variables & Configurations
# ------------------------------------------------------------------------------
VERBOSE=false
PLAYWRIGHT_CMD="npx playwright test"

# ------------------------------------------------------------------------------
# Helper Functions
# ------------------------------------------------------------------------------
# Visual logging utilities (prints to stderr)
msg() {
  echo -e "[$(date +'%Y-%m-%dT%H:%M:%S%z')] $*" >&2
}

log_error() {
  echo -e "\033[0;31m[ERROR]\033[0m $*" >&2
}

log_info() {
  if [ "$VERBOSE" = true ]; then
    echo -e "\033[0;32m[INFO]\033[0m $*" >&2
  fi
}

usage() {
  cat << EOF
Usage: ${SCRIPT_NAME} [-h] [-v]

OPTIONS:
    -h, --help      Display this help menu.
    -v, --verbose   Enable verbose debug logging.

EXAMPLES:
    ./${SCRIPT_NAME} --verbose
EOF
  exit 0
}

# Cleanup routine triggered on script termination or crash
cleanup() {
  trap - SIGINT SIGTERM ERR EXIT
  # Add temporary file or resource removal logic here
  docker stop "${PLAYWRIGHT_DOCKER_SERVER_NAME}"
  if [ "$VERBOSE" = true ]; then
    log_info "Cleanup processes executed successfully."
  fi
}

# Bind the cleanup function to common exit signals
trap cleanup SIGINT SIGTERM ERR EXIT

# ------------------------------------------------------------------------------
# Argument Parsing
# ------------------------------------------------------------------------------
parse_params() {
  while (($# > 0)); do
    case "$1" in
      -h|--help)
        usage
        ;;
      -v|--verbose)
        VERBOSE=true
        shift
        ;;
      --update-snapshots)
        PLAYWRIGHT_CMD="${PLAYWRIGHT_CMD} --update-snapshots"
        shift
        ;;
      --reporter)
        PLAYWRIGHT_CMD="${PLAYWRIGHT_CMD} --reporter $2"
        shift 2
        ;;
      --) # End of all options
        shift
        break
        ;;
      -?*)
        log_error "Unknown option: $1"
        exit 1
        ;;
    esac
  done

  return 0
}

main() {
  log_info "Running in verbose mode."

  PLAYWRIGHT_VERSION="1.61.0"
  HOST_HOSTNAME="hostmachine"
  PLAYWRIGHT_DOCKER_SERVER_NAME="playwright_tests_server"
  log_info "Running playwright's docker server"
  docker run \
    --rm \
    --add-host=${HOST_HOSTNAME}:host-gateway \
    -p 3000:3000 \
    --init \
    -d --name ${PLAYWRIGHT_DOCKER_SERVER_NAME} \
    --workdir /home/pwuser \
    --user pwuser \
    mcr.microsoft.com/playwright:v${PLAYWRIGHT_VERSION}-noble \
    /bin/sh -c "npx -y playwright@${PLAYWRIGHT_VERSION} run-server --port 3000 --host 0.0.0.0"
  log_info "Fetching assets"
  ./fetch_asset.sh
  log_info "Running tests"
  HOSTNAME="${HOST_HOSTNAME}" PW_TEST_CONNECT_WS_ENDPOINT=ws://127.0.0.1:3000/ $PLAYWRIGHT_CMD
}

parse_params "$@"

main "$@"
