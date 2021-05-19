
# Image uploads from Typora into Azure blob storage, using .NET and Rust

*A .NET Core and a Rust application to enable the [Typora](https://typora.io) markdown editor upload images to Azure Blob storage.*

I'm a big fan of Typora, as it has a nice WYSIWYG experience. They also support [custom image uploads](https://support.typora.io/Upload-Image/) to a storage backend of your choice. These samples are small uploader CLIs for Azure blob storage. Essentially, you need to set a `TYPORA_IMAGE_UPLOAD_AZURE_CONNECTION` environment variable, and ensure you have a container named `typoraimages`. 

![Short demo showing an image paste, followed by an upload](https://user-images.githubusercontent.com/795162/118874957-c6f98a00-b8eb-11eb-9fd2-246b8ed0357c.gif)

## Setup

Set the `TYPORA_IMAGE_UPLOAD_AZURE_CONNECTION` environment variable to look like this:

```text
DefaultEndpointsProtocol=https;EndpointSuffix=core.windows.net;AccountName=typora;AccountKey=SomeVerySecretValueYouGetFromPortalDotAzureDotCom==
```

Compile the .NET or Rust bits, go to the Typora `Preferences` page, `Image` section, and in the `Image Upload Settings`, select `Image Uploader: Custom Command`, and set the Command to something like 

```text
"C:\github\chgeuer\TyporaUploaderAzure\rust\target\debug\typora_uploader_azure_blob.exe"
```

So it looks like this

![image-20210519212457278](https://typora.blob.core.windows.net/typoraimages/2021/05/19/19/25/image-20210519212457278----A4PXCWXMWQS25AD1WE2FBXF2HC.png)


## C# and Rust

I started with a .NET version, but also created a Rust version, because I wanted to play with the [Azure for Rust unofficial SDK](https://github.com/Azure/azure-sdk-for-rust/), and spend some more time fighting with the borrow checker 🙄. 

## Security considerations

These utilities generate the MD5 hash of the screenshot or image file you're uploading, and include the hash in the URL. The assumption is that the blob storage container in Azure is public accessible for direct blob read access, but doesn't allow its contents to be listed. I didn't want to include shared access signatures here, thought including the hash in the filename is good enough (for me). 

As a result, the path to the image looks like this:

```https://typora.blob.core.windows.net/typoraimages/2021/05/19/19/25/image-20210519212457278----A4PXCWXMWQS25AD1WE2FBXF2HC.png```

In this URL, you can identify

- `typora` is the storage account name I'm using
- `typoraimages` is the storage container (hardcoded in the apps)
- `2021/05/19/19/25` are year/month/day/hour/minute of the upload
- `image-20210519212457278` is the typora-chosen file name for a bitmap image paste
- `A4PXCWXMWQS25AD1WE2FBXF2HC` is the base32-encoded MD5 hash of the file. I used base32 instead base64 so I could include the full hash in the URL. 

## Feedback and warning

The .NET/C# code certainly isn't something to be ashamed for. The Rust code on the other hand certainly is shitty as hell; I'm barely understanding how to properly fight with the borrow checker, and I certainly don't do idiomatic things, but beginner mistakes in the Rust code. If you are willing to provide feedback, use my github handle `chgeuer` to hit me on [Twitter](https://twitter.com/chgeuer/), or via mail on `microsoft.com`, or open an issue or a PR 😍. 
