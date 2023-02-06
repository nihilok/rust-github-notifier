#!/bin/bash

launchctl unload ~/Library/LaunchAgents/com.gh-notifier.plist
sudo rm /usr/local/bin/gh-notifier 2>/dev/null
sudo rm ~/Library/LaunchAgents/com.gh-notifier.plist 2>/dev/null
echo "gh-notifier has been uninstalled"