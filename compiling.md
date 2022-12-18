
# Compiling

## On WSL2/Ubuntu/ARM

Bring my Windows environment variables down to WSL2:

```batch
setx.exe WSLENV TYPORA_IMAGE_UPLOAD_AZURE_CONNECTION/u:TYPORA_IMAGE_UPLOAD_AZURE_CONTAINER/u:TYPORA_IMAGE_UPLOAD_VANITY_HOST/u
```

Build the ARM bits, and ensure OpenSSL is found:

```shell
#!/bin/bash

sudo apt-get -y install pkg-config libssl-dev

export OPENSSL_LIB_DIR=/usr/lib/aarch64-linux-gnu
export OPENSSL_INCLUDE_DIR=/usr/include/aarch64-linux-gnu/openssl

cargo build --release
```
