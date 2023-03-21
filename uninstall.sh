#!/bin/bash

source ./shared.sh
gh-notifier stop
sudo rm "$LOCAL_BIN" 2>/dev/null
sudo rm ~/Library/LaunchAgents/com.gh-notifier.plist 2>/dev/null
sudo rm /etc/systemd/user/gh-notifier.service 2>/dev/null
sudo rm /etc/systemd/user/gh-notifier.timer 2>/dev/null
echo -e "\n${BOLD}${GREEN_FG}SUCCESS:${DEFAULT} gh-notifier has been uninstalled"
