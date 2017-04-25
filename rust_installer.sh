#!/bin/bash
set -euo pipefail
IFS=$'\n\t'

function yesno() {
	echo "$1"
	select yn in "Yes" "No"; do
		case $yn in
			Yes ) return 0;;
			No  ) return 1;;
		esac
	done
}

function install_rust() {
	yesno "$1. Do you want to try to automatically install it?"

	if which rustup > /dev/null 2>&1; then
		echo "Rustup already present, not installing"
		yesno "Do you want to switch the default toolchain to nightly?"

		echo "Installing the nightly toolchain"
		rustup toolchain install nightly

		echo "Setting default toolchain"
		rustup default nightly
	else
		INPUT_FILE="$(mktemp)"
		trap '{ echo "Removing tmp file"; rm -f \"$INPUT_FILE\" ; exit 255; }' EXIT

		echo "Downloading rustup install script"
		curl https://sh.rustup.rs -sSf > "$INPUT_FILE"

		echo "================================================================================================"
		echo "In the following questions that you may be asked, please choose the Nightly toolchain as default"
		echo "================================================================================================"
		echo
		echo "\"Safely\" executing a file directly downloaded from the Internet"

		sh "$INPUT_FILE"

		trap - EXIT
		echo "Removing tmp file"
		rm -f "$INPUT_FILE"

		echo "Please note that you may need to source ~/.profile / ~/.zprofile,"
		echo "restart your shell or something else for the changes to be taken into account."
	fi

}

echo "Checking if Rust is present"
if which rustc > /dev/null 2>&1; then
	RUST_VERSION="$(rustc --version | awk '{print $2}')"
	SAFE_VERSION="1.18.0"
	if [[ ! "$(printf "%s\n%s" "$RUST_VERSION" "$SAFE_VERSION" | sort -V | head -n1)" =~ $SAFE_VERSION ]]; then
		install_rust "This program wasn't checked with Rust < $SAFE_VERSION (detected version: $RUST_VERSION)"
	else
		echo "Rust is present and has a compatible version"
	fi
else
	install_rust "Rust is not installed"
fi
