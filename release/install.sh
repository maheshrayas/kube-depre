#!/bin/sh

set -e

NAME="kube-depre"
# Get the latest version
KUBEDEPRE_VERSION=$(curl https://api.github.com/repos/maheshrayas/${NAME}/releases/latest | grep "tag_name" | awk '/tag_name/ { gsub(/[",]/,""); print $2}')
DOWNLOAD_URL="https://github.com/maheshrayas/${NAME}/releases/download/${KUBEDEPRE_VERSION}"
TARGET_DIR="${TARGET_DIR:="/usr/local/bin"}"

# Determines OS.
OS="$(uname)"
if [ "${OS}" = "Darwin" ] ; then
  filename="${NAME}-x86_64-apple-darwin.tar.gz"
else
  filename="${NAME}-x86_64-unknown-linux-musl.tar.gz"
fi

echo $DOWNLOAD_URL

printf "\nDownloading %s from %s ...\n" "$NAME" "${DOWNLOAD_URL}/${filename}"

curl -fsLO "${DOWNLOAD_URL}/${filename}"

tar -xzf ${filename}

printf "${NAME} is downloaded at $(pwd)"
