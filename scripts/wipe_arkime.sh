#!/bin/bash

echo "Checking ES/OS..."
until curl -sS "$ARKIME__elasticsearch/_cluster/health?wait_for_status=yellow"
do
    echo "Waiting for ES/OS to start..."
    sleep 1
done

# Wipe is the same initialize except it keeps users intact
echo WIPE | "${ARKIME_INSTALL_DIR}"/db/db.pl "$ARKIME__elasticsearch" wipe
