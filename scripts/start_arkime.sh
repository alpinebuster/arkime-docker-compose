#!/bin/sh

echo "Giving OpenSearch/ElasticSearch time to start..."
echo "Waiting for OS/ES ($ARKIME__elasticsearch) to start..."
until curl -sS "$ARKIME__elasticsearch/_cluster/health?wait_for_status=yellow"
do
    echo "Waiting for OS/ES (http://$ES_OS_USERNAME:$ES_OS_PASSWORD@$ES_OS_HOST:$ES_OS_PORT) to start"
    sleep 1
done
echo "OS/ES ($ARKIME__elasticsearch) successfully started!"

# Configure Arkime to Run
# Ref: https://github.com/arkime/arkime/blob/main/release/Configure
if [ "$INITIALIZE_DB" = "true" ] && [ ! -f "${ARKIME_APP_DIR}/configured" ]; then
    if [ "$PARLIAMENT" = "on" ]; then
        echo "Configuring parliament..."
        "${ARKIME_INSTALL_DIR}/bin/Configure" --parliament
    elif [ "$WISE" = "on" ]; then
        echo "Configuring wise..."
        "${ARKIME_INSTALL_DIR}/bin/Configure" --wise
    elif [ "$CONT3XT" = "on" ]; then
        echo "Configuring cont3xt..."
        "${ARKIME_INSTALL_DIR}/bin/Configure" --cont3xt
    else
        echo "Configuring arkime..."
        "${ARKIME_INSTALL_DIR}/bin/Configure"
    fi

    touch "${ARKIME_APP_DIR}/configured"
fi

# Give option to init ElasticSearch
if [ "$INITIALIZE_DB" = "true" ] ; then
    echo "Init elasticsearch and then create an admin..."

    if [ ! -f $ARKIME_INSTALL_DIR/etc/.initialized ]; then
        echo INIT | $ARKIME_INSTALL_DIR/db/db.pl "$ARKIME__elasticsearch" init
        $ARKIME_INSTALL_DIR/bin/arkime_add_user.sh "$ARKIME_USERNAME" "Admin User" $ARKIME_PASSWORD --admin
        echo $ARKIME_VERSION > $ARKIME_INSTALL_DIR/etc/.initialized
    else
        # possible update
        read old_ver < $ARKIME_INSTALL_DIR/etc/.initialized
        # detect the newer version
        newer_ver=`echo -e "$old_ver\n$ARKIME_VERSION" | sort -rV | head -n 1`
        # the old version should not be the same as the newer version
        # otherwise -> upgrade
        if [ "$old_ver" != "$newer_ver" ]; then
            echo "Upgrading OS database..."
            $ARKIME_INSTALL_DIR/db/db.pl "$ARKIME__elasticsearch" upgradenoprompt
            echo $ARKIME_VERSION > $ARKIME_INSTALL_DIR/etc/.initialized
        fi
    fi

    echo "Finished the DB initializing."
    exit 0
fi

# Give option to wipe ElasticSearch
if [ "$WIPE_DB" = "true" ]; then
    "${ARKIME_APP_DIR}/wipe_arkime.sh"
fi

echo "Look at log files at below for errors information:"
echo "  /opt/arkime/logs/*.log"

if [ "$CAPTURE" = "on" ]; then
    echo "Launching capture..."
    # Ensure "$ARKIME_INSTALL_DIR/raw" directory is writable for user 'nobody' (used by the capture process)
    chmod -R 777 "$ARKIME_INSTALL_DIR/raw"
    "$ARKIME_APP_DIR/docker.sh" capture --forever --config "$ARKIME_INSTALL_DIR/etc/config.ini" | tee -a "$ARKIME_INSTALL_DIR/logs/capture.log" 2>&1 &
fi

if [ "$CONT3XT" = "on" ]; then
    echo "Launching cont3xt..."
    echo "Visit http://127.0.0.1:3218 with your favorite browser."
    echo "  User    : $ARKIME_USERNAME"
    echo "  Password: $ARKIME_PASSWORD"
    "$ARKIME_APP_DIR/docker.sh" cont3xt --forever --config "$ARKIME_INSTALL_DIR/etc/cont3xt.ini" | tee -a "$ARKIME_INSTALL_DIR"/logs/cont3xt.log 2>&1 &
fi

if [ "$PARLIAMENT" = "on" ]; then
    echo "Launching parliament..."
    echo "Visit http://127.0.0.1:8008 with your favorite browser."
    echo "  User    : $ARKIME_USERNAME"
    echo "  Password: $ARKIME_PASSWORD"
    "$ARKIME_APP_DIR/docker.sh" parliament --forever --config "$ARKIME_INSTALL_DIR/etc/parliament.ini" | tee -a "$ARKIME_INSTALL_DIR/logs/parliament.log" 2>&1 &
fi

if [ "$WISE" = "on" ]; then
    echo "Launching wise..."
    echo "Accessible via http://127.0.0.1:8081 or http://arkime-wise:8081."
    "$ARKIME_APP_DIR/docker.sh" wise --forever --config "$ARKIME_INSTALL_DIR/etc/wise.ini" | tee -a "$ARKIME_INSTALL_DIR"/logs/wise.log 2>&1 &
fi

if [ "$VIEWER" = "on" ]; then
    echo "Launching viewer..."
    echo "Visit http://127.0.0.1:8005 with your favorite browser."
    echo "  User    : $ARKIME_USERNAME"
    echo "  Password: $ARKIME_PASSWORD"
    "$ARKIME_APP_DIR/docker.sh" viewer --forever --config "$ARKIME_INSTALL_DIR/etc/config.ini" | tee -a "$ARKIME_INSTALL_DIR/logs/viewer.log" 2>&1 &
fi

wait
Echo "Job done!"
