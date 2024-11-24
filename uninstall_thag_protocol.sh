#!/bin/bash
# uninstall_thag_protocol.sh

set -e

APP_NAME="ThagHandler"

echo "This will uninstall the $APP_NAME protocol handler"
read -p "Continue? (y/N) " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    if [ -d "/Applications/$APP_NAME.app" ]; then
        echo "Removing $APP_NAME.app..."
        sudo rm -rf "/Applications/$APP_NAME.app"
        
        # Force Launch Services to rebuild its database
        /System/Library/Frameworks/CoreServices.framework/Frameworks/LaunchServices.framework/Support/lsregister -kill -r -domain local -domain system -domain user
        
        echo "Uninstallation completed successfully"
    else
        echo "Error: $APP_NAME.app not found in /Applications"
        exit 1
    fi
else
    echo "Uninstallation cancelled"
    exit 1
fi
