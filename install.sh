#!/bin/bash
set -e

# check working directory is script directory
[ ! -f "$(pwd)/install.sh" ] && [ ! -f "$(pwd)/shared.sh" ] &&
  echo -e "
install.sh must be run from inside the source directory
" && exit 1

source ./shared.sh

# check for correct environment variable
[ -z "$GH_NOTIFIER_TOKEN" ] &&
  echo -e "
${ERROR}ERROR${DEFAULT} GH_NOTIFIER_TOKEN environment variable not set
${YELLOW_FG}hint:${DEFAULT} prefix the command with 'GH_NOTIFIER_TOKEN=<personal-access-token-with-notifications-scope>'
" && exit 1

# build latest version of rust binary
if command -v cargo -h &>/dev/null; then
  echo "Building latest binary"
  cargo build --release
  rm "$BINARY_PATH" &>/dev/null || true
  mkdir -p "$DIST_PATH"
  mv target/release/gh-notifier "$DIST_PATH"
  chmod +x "$BINARY_PATH"
  COMPILED=true
else
  echo -e "${ERROR}ERROR${DEFAULT} cargo must be installed to build rust binary https://doc.rust-lang.org/cargo/getting-started/installation.html" && exit 1
fi

# create symbolic link on path
LOCAL_BIN=$HOME/.local/bin
mkdir -p "${LOCAL_BIN}" || exit 1
rm "${LOCAL_BIN}"/gh-notifier &>/dev/null || true
ln -s "${BINARY_PATH}" "${LOCAL_BIN}"/gh-notifier &>/dev/null

if [[ $OSTYPE == 'darwin'* ]]; then
  # install terminal-notifier if not already installed and homebrew is available
  if ! command -v terminal-notifier &>/dev/null && command -v brew -h &>/dev/null; then brew install terminal-notifier; fi

  # unload existing launchd service
  PLIST_PATH=$HOME/Library/LaunchAgents/com.gh-notifier.plist
  launchctl unload "${PLIST_PATH}" 2>/dev/null

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
" >"${PLIST_PATH}"
  # start launchd service
  load_service() {
    launchctl load "${PLIST_PATH}"
  }
else
  if ! command -v systemd &>/dev/null; then echo "systemd is not available; skipping linux service installation"; fi
  if ! sudo -n true 2>/dev/null; then
    echo "Password required to enable systemd service"
  fi
  echo "[Unit]
Description=Github Notifier

[Service]
WorkingDirectory=$HOME
ExecStart=$LOCAL_BIN/gh-notifier
Environment=GH_NOTIFIER_TOKEN=$GH_NOTIFIER_TOKEN
Environment=DISPLAY=:0
Type=oneshot
" >>gh-notifier.service && sudo mv gh-notifier.service /etc/systemd/user/

  echo "[Unit]
Description=Github Notifier

[Timer]
OnUnitActiveSec=30s
OnBootSec=30s

[Install]
WantedBy=timers.target
" >>gh-notifier.timer && sudo mv gh-notifier.timer /etc/systemd/user/

  systemctl --user stop gh-notifier.timer &> /dev/null || true

  load_service() {
    systemctl --user daemon-reload && systemctl --user start gh-notifier.timer && systemctl --user enable gh-notifier.timer
  }
fi

if load_service; then
  OUTPUT="
${SUCCESS}SUCCESS${DEFAULT} ${BOLD}gh-notifier${DEFAULT} is now running...

${YELLOW_FG}Use${DEFAULT}${BOLD} \`gh-notifier stop\` ${DEFAULT}${YELLOW_FG}to stop the service,
and ${DEFAULT}${BOLD}\`gh-notifier start\` ${DEFAULT}${YELLOW_FG}to restart it.${DEFAULT}"
  echo -e "${OUTPUT}"
else
  echo -e "${ERROR}ERROR${DEFAULT} could not load service"
fi
