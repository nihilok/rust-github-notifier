# gh-notifier: GitHub Pull Request Notifications

## Prerequisites:

- depends on `terminal-notifier` [source](https://github.com/julienXX/terminal-notifier) (macos) and this will be installed automatically if you use the install script provided. (On linux the equivalent dependency is `notify-send` which will not be installed automatically).
- installation (or standalone binary) depends on having a `GH_NOTIFIER_TOKEN` environment variable set to a [personal access token](https://github.com/settings/tokens) with Notifications scope.

### External Crates:

- https://github.com/nihilok/rust-notify
- https://github.com/nihilok/rust-command-line

## Installation:

```bash
GH_NOTIFIER_TOKEN=<INSERT TOKEN HERE> ./install.sh
```

_The token will be stored in the launchd plist xml and the install script can be rerun if the token changes, or alternatively the variable in `~/Library/LaunchAgents/com.gh-notifier.plist` can be manually updated._

## Uninstallation:

```bash
./uninstall.sh
```

## Usage:

Once installed, the daemon, managed by launchd, polls the GitHub notifications API at 30 second intervals and displays desktop notifications for any new GitHub notifications received.

Without installation, the binary in `dist/` can be used for a one-off call, or be scheduled at the desired interval with crontab. Bear in mind that both the `GH_NOTIFIER_TOKEN` and a `$PATH` variable with `/opt/homebrew/bin/` will need to be available.
