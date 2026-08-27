#!/bin/bash

# --- Strict / Safe Mode ---
# -e: Exit immediately if a command exits with a non-zero status.
# -u: Treat unset variables as an error and exit immediately.
# -o pipefail: Prevent errors in a pipeline from being masked.
set -euo pipefail
cd "$(dirname -- "${BASH_SOURCE[0]}")" || exit 1

ARTIFACT_NAME=screenshots
ARTIFACT_PATH=tests/__screenshots__

FULL_BRANCH_NAME=$( git rev-parse --abbrev-ref --symbolic-full-name @{u} )
BRANCH_NAME=${FULL_BRANCH_NAME#origin/}

RUN_ID=$( gh api repos/DouglasMeyer/ledger_oxide/actions/runs?branch=${BRANCH_NAME} --jq '.workflow_runs[0].id' )

rm -rf ${ARTIFACT_PATH} || true
mkdir ${ARTIFACT_PATH}
cd ${ARTIFACT_PATH}
gh run download ${RUN_ID} -n ${ARTIFACT_NAME}
cd - > /dev/null