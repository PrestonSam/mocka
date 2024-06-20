#!/usr/bin/zsh

PEST_FILE="$1"
TARGET_FILE="$2"


{ echo 'ba\nd WHITESPACE\nr body\n'; yes c; } \
	| pest_debugger -g "$PEST_FILE" -i "$TARGET_FILE" \
	| tee >(grep -m 1 'Error: End-of-input reached' > /dev/null && "$(pkill pest_debugger)") \
	| grep -v 'Error: End-of-input reached'
