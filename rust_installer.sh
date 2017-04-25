#!/bin/bash
set -euo pipefail
IFS=$'\n\t'

echo "Checking if Rust is present"
if which rustc > /dev/null 2>&1; then
	RUST_VERSION="$(rustc --version | awk '{print $2}')"
	SAFE_VERSION="1.18.0"
	if [[ ! "$(printf "%s\n%s" "$RUST_VERSION" "$SAFE_VERSION" | sort -V | head -n1)" =~ $SAFE_VERSION ]]; then
		echo "Warning: this program wasn't checked with Rust < $SAFE_VERSION"
	else
		echo "Rust is present and has a compatible version"
	fi
else
	if dialog --yesno "Rust is not installed. Do you want to try to automatically install it?" 10 40; then
		INPUT_FILE="$(mktemp)"
		dialog --checklist "Which version do you want to install?" 15 50 15 "nightly" "The latest of the latest" on "beta" "The beta channel" off "stable" "Boring version" off 2> "$INPUT_FILE"

		IFS=" "
		CHANNELS=( $(cat "$INPUT_FILE") )
		IFS=$'\n\t'

		if [ "${#CHANNELS[@]}" -gt 1 ]; then
			CHOICES=()

			CHOICES+=( "${CHANNELS[0]}" "Still ${CHANNELS[0]}" on )
			for CHANNEL in "${CHANNELS[@]:1}"; do CHOICES+=( "$CHANNEL" "Still $CHANNEL" off ); done

			dialog --radiolist "Which channel should be the default?" 15 50 15 "${CHOICES[@]}" 2> "$INPUT_FILE"
			DEFAULT_CHANNEL="$(cat "$INPUT_FILE")"
		elif [ "${#CHANNELS[@]}" -eq 0 ]; then
			dialog --msgbox "Not installing Rust, I guess…" 5 40
			exit 1
		else
			DEFAULT_CHANNEL=${CHANNELS[0]}
		fi

		clear
		echo "Downloading rustup"
		curl https://sh.rustup.rs -sSf > "$INPUT_FILE"
		echo "\"Safely\" executing a file directly download from the Internet"
		sh "$INPUT_FILE"

		echo "Installing toolchains"
		for CHANNEL in "${CHANNELS[@]}"; do 
			rustup toolchain install "$CHANNEL"
		done

		echo "Setting default toolchain"
		rustup default "$DEFAULT_CHANNEL"

		rm "$INPUT_FILE"
	fi
fi
