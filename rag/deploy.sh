#!/bin/bash

# Deploy to Nexus Artifactory
# curl -v -u username:password! --upload-file pkg/lib_presets_bg.wasm https://artifactory.pegadaian.co.id/repository/[dir]/lib_presets_050625.wasm

curl --location --request PUT 'http://artifactory.pegadaian.co.id/repository/[PATH]' \
--header 'Authorization: Basic [BASIC_AUTH]' \
--form '=@"/C:/Users/[DIR]/my-wasm-lib/pkg/my_wasm_lib_bg.wasm"'
