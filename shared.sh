LOCAL_BIN=$HOME/.local/bin
DEFAULT=$(tput sgr0)
BOLD=$(tput bold)
RED_BG=$(tput setab 1)
BLACK_FG=$(tput setaf 0)
YELLOW_FG=$(tput setaf 3)
GREEN_FG=$(tput setaf 2)
ERROR="${RED_BG}${BLACK_FG}${BOLD}"

BASE_PATH=$(pwd)
DIST_PATH="$BASE_PATH/dist"
SCRIPT_PATH="$BASE_PATH/install.sh"
BINARY_PATH="$DIST_PATH/gh-notifier"