LOCAL_BIN=$HOME/.local/bin
DEFAULT=$(tput sgr0)
BOLD=$(tput bold)
RED_BG=$(tput setab 1)
BLACK_FG=$(tput setaf 0)
YELLOW_FG=$(tput setaf 3)
GREEN_FG=$(tput setaf 2)
GREEN_BG=$(tput setab 2)
ERROR="${RED_BG}${BLACK_FG}${BOLD}"
SUCCESS="${GREEN_BG}${BLACK_FG}${BOLD}"

BASE_PATH=$(pwd)
DIST_PATH="$BASE_PATH/dist"
SCRIPT_PATH="$BASE_PATH/install.sh"
BINARY_PATH="$DIST_PATH/gh-notifier"

sudo rm "$LOCAL_BIN/gh-notifier" 2>/dev/null || true
sudo rm ~/Library/LaunchAgents/com.gh-notifier.plist 2>/dev/null || true
sudo rm /etc/systemd/user/gh-notifier.service 2>/dev/null || true
sudo rm /etc/systemd/user/gh-notifier.timer 2>/dev/null || true