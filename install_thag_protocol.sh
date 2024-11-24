#!/bin/bash
# install_thag_protocol.sh

set -e  # Exit on any error

# Configuration
BUNDLE_ID="io.github.durbanlegend.thaghandler"
COMPANY_NAME="durbanlegend"
APP_NAME="ThagHandler"
PROTOCOL="thag"
THAG_PATH="/usr/local/bin/thag"

# Function to check if thag is installed
check_thag() {
    if ! command -v thag >/dev/null 2>&1; then
        echo "Error: thag is not installed or not in PATH"
        echo "Expected location: $THAG_PATH"
        exit 1
    fi
}

# Function to check if app bundle already exists
check_existing() {
    if [ -d "/Applications/$APP_NAME.app" ]; then
        echo "Warning: $APP_NAME.app already exists in /Applications"
        read -p "Do you want to replace it? (y/N) " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            echo "Installation cancelled"
            exit 1
        fi
        sudo rm -rf "/Applications/$APP_NAME.app"
    fi
}

# Main installation function
install_handler() {
    echo "Installing $APP_NAME protocol handler..."

    # Create a temporary directory
    TEMP_DIR=$(mktemp -d)
    if [ ! -d "$TEMP_DIR" ]; then
        echo "Error: Failed to create temporary directory"
        exit 1
    fi

    APP_BUNDLE="$TEMP_DIR/$APP_NAME.app"
    CONTENTS_DIR="$APP_BUNDLE/Contents"
    MACOS_DIR="$CONTENTS_DIR/MacOS"
    RESOURCES_DIR="$CONTENTS_DIR/Resources"

    # Create directory structure
    mkdir -p "$MACOS_DIR" "$RESOURCES_DIR"

    # Create Info.plist
    cat > "$CONTENTS_DIR/Info.plist" << EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleIdentifier</key>
    <string>$BUNDLE_ID</string>
    <key>CFBundleName</key>
    <string>$APP_NAME</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleExecutable</key>
    <string>thag_launcher</string>
    <key>LSBackgroundOnly</key>
    <false/>
    <key>CFBundleURLTypes</key>
    <array>
        <dict>
            <key>CFBundleURLName</key>
            <string>Thag Protocol</string>
            <key>CFBundleURLSchemes</key>
            <array>
                <string>$PROTOCOL</string>
            </array>
        </dict>
    </array>
</dict>
</plist>
EOF

# Create launcher script
cat > "$MACOS_DIR/thag_launcher" << EOF
#!/usr/bin/osascript

on run argv
log "Launcher started"

-- Get the URL from the open location event
on open location this_URL
    log "Received URL: " & this_URL

    -- Strip the protocol prefix
    set stripped_URL to text 7 thru -1 of this_URL -- removes "thag://"

    -- Construct the shell command
    set cmd to "echo 'Running thag with URL: " & stripped_URL & "' && $THAG_PATH -u '" & stripped_URL & "' && echo 'Press any key to close' && read -n 1"

    tell application "Terminal"
        activate
        do script cmd
    end tell
end open location

log "Launcher completed"
end run

-- Handle the open location event
on open location this_URL
log "Direct open location event: " & this_URL
run {this_URL}
end open location
EOF

# Change the file extension to .scpt
mv "$MACOS_DIR/thag_launcher" "$MACOS_DIR/thag_launcher.scpt"

# Create a shell script wrapper
cat > "$MACOS_DIR/thag_launcher" << EOF
#!/bin/bash
exec osascript "$MACOS_DIR/thag_launcher.scpt" "\$@"
EOF

    # Set permissions
    chmod +x "$MACOS_DIR/thag_launcher"

    # Move to Applications
    if ! sudo mv "$APP_BUNDLE" /Applications/; then
        echo "Error: Failed to move app bundle to /Applications"
        rm -rf "$TEMP_DIR"
        exit 1
    fi

    # Set ownership (important for permissions)
    sudo chown -R $(whoami) "/Applications/$APP_NAME.app"

    # Register with Launch Services
    if ! /System/Library/Frameworks/CoreServices.framework/Frameworks/LaunchServices.framework/Support/lsregister -R "/Applications/$APP_NAME.app"; then
        echo "Warning: Failed to register with Launch Services"
    fi

    # Cleanup
    rm -rf "$TEMP_DIR"

    echo "Installation completed successfully!"
    echo "You can now use $PROTOCOL:// links in your browser"
    echo "Check /tmp/thag_launcher.log for debugging information"
}

# Main execution
echo "This script will install the $APP_NAME protocol handler"
echo "It will require sudo access to install to /Applications"
read -p "Continue? (y/N) " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    check_thag
    check_existing
    install_handler
else
    echo "Installation cancelled"
    exit 1
fi
