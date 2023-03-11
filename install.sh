#!/bin/bash
set -e
DEFAULT=$(tput sgr0)
BOLD=$(tput bold)
RED_BG=$(tput setab 1)
BLACK_FG=$(tput setaf 0)
YELLOW_FG=$(tput setaf 3)
GREEN_FG=$(tput setaf 2)
ERROR="${RED_BG}${BLACK_FG}${BOLD}"

BASE_PATH=$(pwd)
DIST_PATH="$BASE_PATH/dist"
BINARY_PATH="$DIST_PATH/gh-notifier"

# check working directory is script directory
[ ! -d "$DIST_PATH" ] &&
echo -e "
${ERROR}ERROR:${DEFAULT} install.sh must be run from inside the source directory
" && exit 1

# check for correct environment variable
[ -z "$GH_NOTIFIER_TOKEN" ] &&
echo -e "
${ERROR}ERROR:${DEFAULT} GH_NOTIFIER_TOKEN environment variable not set
${YELLOW_FG}hint:${DEFAULT} prefix the command with 'GH_NOTIFIER_TOKEN=<personal-access-token-with-notifications-scope>'
" && exit 1

# build latest version of rust binary
if command -v cargo -h &> /dev/null; then
  echo "Building latest binary"
  cargo build --release
  mv target/release/gh-notifier dist
  COMPILED=true
fi

# install terminal-notifier if not already installed
if ! command -v terminal-notifier &> /dev/null; then brew install terminal-notifier; fi

# unload existing launchd service
PLIST_PATH=$HOME/Library/LaunchAgents/com.gh-notifier.plist
launchctl unload "${PLIST_PATH}" 2>/dev/null

# create symbolic link on path
LOCAL_BIN=$HOME/.local/bin
mkdir -p "${LOCAL_BIN}" || exit 1
rm "${LOCAL_BIN}"/gh-notifier 2>/dev/null || true
ln -s "${BINARY_PATH}" "${LOCAL_BIN}"/gh-notifier

# create launch agent plist file
echo "\
<?xml version=\"1.0\" encoding=\"UTF-8\"?>
<!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">
<plist version=\"1.0\">
    <dict>
        <key>Label</key>
        <string>local.gh-notifier</string>
        <key>Program</key>
        <string>/usr/local/bin/gh-notifier</string>
        <key>EnvironmentVariables</key>
        <dict>
            <key>PATH</key>
            <string>$HOME/.local/bin:/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin:/opt/homebrew/bin</string>
            <key>GH_NOTIFIER_TOKEN</key>
            <string>$GH_NOTIFIER_TOKEN</string>
        </dict>
        <key>RunAtLoad</key>
        <true/>
        <key>StartInterval</key>
        <integer>30</integer>
    </dict>
</plist>
" > "${PLIST_PATH}"

# start launchd service
if launchctl load "${PLIST_PATH}"; then
OUTPUT="${GREEN_FG}${BOLD}gh-notifier${DEFAULT} is now running...

${YELLOW_FG}Use${DEFAULT}${BOLD} \`gh-notifier stop\` ${DEFAULT}${YELLOW_FG}to stop the service${DEFAULT}"
[[ -n ${COMPILED} ]] && OUTPUT="\t${OUTPUT}"
echo -e "
${OUTPUT}"
else
  echo -e "${RED_BG}${BLACK_FG}ERROR:${DEFAULT} could not load launchd service"
fi
