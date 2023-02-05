#!/bin/bash

echo "Installing rust-macos-github-notifier"
mkdir -p /usr/local/bin || exit 1;
sudo rm /usr/local/bin/gh-notifier || echo "Creating new symlink in /usr/local/bin";
sudo ln -s "$(cd "$(dirname -- "dist/gh-notifier")" >/dev/null; pwd -P)/$(basename -- "dist/gh-notifier")" /usr/local/bin/gh-notifier;
cp gh-notifier.plist ~/Library/LaunchAgents/gh-notifier.plist
launchctl unload ~/Library/LaunchAgents/gh-notifier.plist || true;
launchctl load ~/Library/LaunchAgents/gh-notifier.plist
echo "Done. Run \`gh-notifier\` to poll the GitHub API for notifications once. Remember to set the \`GH_NOTIFIER_TOKEN\` environment variable. For continual polling, add to crontab at desired interval. You may need to set the PATH variable within the crontab."
