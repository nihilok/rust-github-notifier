#!/bin/bash

PLIST_PATH=$HOME/Library/LaunchAgents/com.gh-notifier.plist
launchctl unload "$PLIST_PATH" 2>/dev/null
mkdir -p /usr/local/bin || exit 1;
sudo rm /usr/local/bin/gh-notifier 2>/dev/null
sudo ln -s "$(cd "$(dirname -- "dist/gh-notifier")" >/dev/null; pwd -P)/$(basename -- "dist/gh-notifier")" /usr/local/bin/gh-notifier;
echo "
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
            <string>/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin:/opt/homebrew/bin</string>
            <key>GH_NOTIFIER_TOKEN</key>
            <string>$GH_NOTIFIER_TOKEN</string>
        </dict>
        <key>RunAtLoad</key>
        <true/>
        <key>StartInterval</key>
        <integer>30</integer>
    </dict>
</plist>
" > "$PLIST_PATH"
if launchctl load "$PLIST_PATH" ; then
echo "gh-notifier is now running...
Use 'launchctl unload $PLIST_PATH' to stop the service"
else
  echo "failed to load launchd daemon; run program manually with 'gh-notifier' or add to crontab for scheduling"
fi
