#!/usr/bin/zsh

let PEST_FILE="$1"
let TARGET_FILE="$2"


{ echo 'ba\nd WHITESPACE\nd TAB\nr body\n'; yes c } \
	| pest_debugger -g "$PEST_FILE" -i "$TARGET_FILE" \
	| tee >(grep -m 1 'Error: End-of-input reached' > /dev/null && "$(pkill pest_debugger)") \
	| grep -v 'Error: End-of-input reached'
