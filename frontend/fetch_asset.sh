#!/bin/bash

# --- Strict / Safe Mode ---
# -e: Exit immediately if a command exits with a non-zero status.
# -u: Treat unset variables as an error and exit immediately.
# -o pipefail: Prevent errors in a pipeline from being masked.
set -euo pipefail
cd "$(dirname -- "${BASH_SOURCE[0]}")" || exit 1

# Globals and variables
SCRIPT_NAME=$(basename "${BASH_SOURCE[0]}")
ARTIFACT_NAME=screenshots
ARTIFACT_PATH=tests/__screenshots__

FULL_BRANCH_NAME=$( git rev-parse --abbrev-ref --symbolic-full-name @{upstream} || echo "main" )
BRANCH_NAME=${FULL_BRANCH_NAME#origin/}

# --- Help / Usage Documentation ---
usage() {
    cat << EOF
Usage: ${SCRIPT_NAME} [OPTIONS] [branch_name]

Fetch github workflow screenshots to serve as test standards.

Options:
  -h, --help      Display this help message and exit.
  branch_name     Branch to fetch screenshots from, defaults to upstream branch or "main"

Example:
  ./${SCRIPT_NAME} [branch_name]
EOF
    exit 0
}

parse_params() {
    # Loop through arguments using a while-case matrix
    while [[ $# -gt 0 ]]; do
        case "$1" in
            -h|--help)
                usage
                ;;
            -*)
                echo "Unknown option: $1" >&2
                echo "Try '${SCRIPT_NAME} --help' for more information."
                exit 1
                ;;
            *)
                BRANCH_NAME=$1
                shift
                ;;
        esac
    done
}

parse_params "$@"

RUN_ID=$( gh api repos/DouglasMeyer/ledger_oxide/actions/runs?branch=${BRANCH_NAME} --jq '.workflow_runs[0].id' )

rm -rf ${ARTIFACT_PATH} || true
mkdir ${ARTIFACT_PATH}
cd ${ARTIFACT_PATH}
gh run download ${RUN_ID} -n ${ARTIFACT_NAME}
cd - > /dev/null