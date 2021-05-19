
# Image uploads from Typora into Azure blob storage

I'm a big fan of the [Typora](https://typora.io) markdown editor, as it has a nice WYSIWYG experience. They also support [custom image uploads](https://support.typora.io/Upload-Image/) to a storage backend of your choice.

This sample here is a small uploader CLI for Azure blob storage. Essentially, you need to set a `TYPORA_IMAGE_UPLOAD_AZURE_CONNECTION` environment variable, and ensure you have a container named `typoraimages`. 

## C# and Rust

I started with a .NET version, but also created a Rust version, because I wanted to play with the [Azure for Rust unofficial SDK](https://github.com/Azure/azure-sdk-for-rust/), and spend some more time fighting with the borrow checker 🙄. 

## Security considerations

The utilities generate the MD5 hash of the screenshot or image file you're uploading, and include it in the filename. The assumption is that the blob storage container in Azure is public accessible for direct blob read access, but doesn't allow its contents to be listed. I didn't want to include shared access signatures here, thought including the hash in the filename is good enough (for me). 

As a result, the path to the image looks like this:

```https://typora.blob.core.windows.net/typoraimages/2021/05/19/18/57/image-20210519205709071----TNDBDDHB60RES85XW44SS0WPWG.png```

- `typora` is the storage account name I'm using
- `typoraimages` is the storage container (hardcoded in the apps)
- `2021/05/19/18/57` are year/month/day/hour/minute of the upload
- `image-20210519205709071` is the typora-chosen file name for an image paste
- `TNDBDDHB60RES85XW44SS0WPWG` is the base32-encoded MD5 hash of the file

