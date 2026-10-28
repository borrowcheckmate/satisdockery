#!/bin/bash

if [ "$(uname)" != "Linux" ]; then
    echo "Only Linux runtime is supported"
    exit 1
fi

arch=$(uname -m)
[[ -z "${SATISDOCKERY_PATH}" ]] && run_directory='~/satisdockery' || run_directory="${SATISDOCKERY_PATH}"
satisdockery -p $run_directory

if [ "$arch" == "x86_64" ]
then
    $run_directory/FactoryServer.sh
else
    box64 $run_directory/FactoryServer.sh
fi
