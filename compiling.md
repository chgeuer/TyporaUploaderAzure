
# Compiling

## On WSL2/Ubuntu/ARM

```batch
setx.exe WSLENV TYPORA_IMAGE_UPLOAD_AZURE_CONNECTION/u:TYPORA_IMAGE_UPLOAD_AZURE_CONTAINER/u:TYPORA_IMAGE_UPLOAD_VANITY_HOST/u
```

use url::Url;



```shell
OPENSSL_LIB_DIR=/usr/lib/aarch64-linux-gnu OPENSSL_INCLUDE_DIR=/usr/include/aarch64-linux-gnu/openssl cargo build --release
```
