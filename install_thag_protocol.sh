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
#!/bin/bash
url="\$1"
if [ -z "\$url" ]; then
echo "Error: No URL provided"
exit 1
fi
url=\${url#$PROTOCOL://}
osascript -e 'tell application "WezTerm"
activate
do script "echo \"Running thag with URL: \$url\" && $THAG_PATH -u \"\$url\" ; echo \"Press any key to close\"; read -n 1"
end tell'
EOF

    # Set permissions
    chmod +x "$MACOS_DIR/thag_launcher"

    # Move to Applications
    if ! sudo mv "$APP_BUNDLE" /Applications/; then
        echo "Error: Failed to move app bundle to /Applications"
        rm -rf "$TEMP_DIR"
        exit 1
    fi

    # Register with Launch Services
    if ! /System/Library/Frameworks/CoreServices.framework/Frameworks/LaunchServices.framework/Support/lsregister -R "/Applications/$APP_NAME.app"; then
        echo "Warning: Failed to register with Launch Services"
    fi

    # Cleanup
    rm -rf "$TEMP_DIR"

    echo "Installation completed successfully!"
    echo "You can now use $PROTOCOL:// links in your browser"
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
