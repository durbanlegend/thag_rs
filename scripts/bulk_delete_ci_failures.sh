#!/bin/bash

# Set your repository owner and name
OWNER="durbanlegend"
REPO="thag_rs"

# Get a list of failed workflow run IDs
# The '--json databaseId' extracts the run ID, and '-q '.[].databaseId'' filters for only the IDs.
# '--jq '.[].databaseId'' uses the built-in gojq for filtering.
FAILED_RUN_IDS=$(gh run list --repo "$OWNER/$REPO" --status failure --json databaseId --jq '.[].databaseId')

# Check if there are any failed runs to delete
if [ -z "$FAILED_RUN_IDS" ]; then
  echo "No failed workflow runs found to delete."
  exit 0
fi

echo "Deleting the following failed workflow runs in $OWNER/$REPO:"
echo "$FAILED_RUN_IDS"

# Iterate through the failed run IDs and delete each one
for RUN_ID in $FAILED_RUN_IDS; do
  echo "Deleting run ID: $RUN_ID"
  gh run delete "$RUN_ID" --repo "$OWNER/$REPO"
done

echo "Bulk deletion of failed workflow runs complete."
