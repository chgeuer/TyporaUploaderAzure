namespace TyporaUploaderAzure
{
    using System;
    using System.IO;
    using System.Linq;
    using System.Security.Cryptography;
    using System.Text;
    using System.Threading.Tasks;
    using System.Web;
    using Azure.Storage.Blobs.Models;
    using SimpleBase;

    class Program
    {
        // https://support.typora.io/Upload-Image/
        static async Task Main(string[] args)
        {
            var connectionString = Environment.GetEnvironmentVariable("TYPORA_IMAGE_UPLOAD_AZURE_CONNECTION");
            var serviceClient = new Azure.Storage.Blobs.BlobServiceClient(connectionString: connectionString);
            var containerClient = serviceClient.GetBlobContainerClient(blobContainerName: "typoraimages");
            // await containerClient.CreateIfNotExistsAsync(Azure.Storage.Blobs.Models.PublicAccessType.Blob);

            var prefix = DateTime.Now.ToString("yyyy/MM/dd/HH/mm");

            var tasks = args.Select(async filename =>
            {
                filename = HttpUtility.UrlDecode(filename);
            
                var fi = new FileInfo(filename);
                var bytes = await File.ReadAllBytesAsync(path: fi.FullName);
                using var hashAlgo = MD5.Create();

                var hash = hashAlgo.ComputeHash(bytes);
                var hashBase32 = Base32.Crockford.Encode(hash);
                var fileWithoutExtension = fi.Name.Substring(0, fi.Name.Length - fi.Extension.Length);
                var blobName = $"{prefix}/{fileWithoutExtension}----{hashBase32}{fi.Extension}";
                var blobClient = containerClient.GetBlobClient(blobName);

                string mimeType(string extension) => extension switch
                {
                    ".png" => "image/png",
                    ".jpeg" => "image/jpeg",
                    ".jpg" => "image/jpeg",
                    _ => "application/octet-stream",
                };

                if (!await blobClient.ExistsAsync())
                {
                    using var ms = new MemoryStream(bytes);
                    await blobClient.UploadAsync(ms);

                    var headers = new BlobHttpHeaders
                    {
                        ContentType = mimeType(fi.Extension),
                        ContentHash = hash,
                        CacheControl = "max-age=31536000",
                    };

                    await blobClient.SetHttpHeadersAsync(headers);
                }

                return blobClient.Uri.AbsoluteUri;
            });

            await Task.WhenAll(tasks);

            tasks
                .Select(t => t.Result)
                .ToList()
                .ForEach(Console.WriteLine);
        }
    }
}