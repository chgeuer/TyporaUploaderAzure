use azure_core::prelude::*;
use azure_storage::blob::prelude::*;
use azure_storage::core::prelude::*;
use chrono::Utc;
use std::env;
use std::error::Error;
use std::ffi::OsStr;
use std::path::Path;
use std::sync::Arc;
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use url::Url;

fn get_mimetype_from_filename(filename: &str) -> &str {
    // could also use mime_guess = "2.0.3"
    match Path::new(filename).extension().and_then(OsStr::to_str) {
        Some("jpg") => "image/jpeg",
        Some("jpeg") => "image/jpeg",
        Some("png") => "image/png",
        Some("svg") => "image/svg+xml",
        _ => "application/octet-stream",
    }
}

// "C:\github\chgeuer\TyporaUploaderAzure\dotnet\bin\Debug\netcoreapp3.1\TyporaUploaderAzure.exe"
// "C:\github\chgeuer\TyporaUploaderAzure\rust\target\debug\typora_uploader_azure_blob.exe"
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let connection_string = std::env::var("TYPORA_IMAGE_UPLOAD_AZURE_CONNECTION")
        .expect("Set env variable TYPORA_IMAGE_UPLOAD_AZURE_CONNECTION first!");

    let cs = azure_storage::ConnectionString::new(&connection_string).unwrap();

    let http_client: Arc<Box<dyn HttpClient>> = Arc::new(Box::new(reqwest::Client::new()));

    let container_name: String = "typoraimages".to_owned();

    let storage_account =
        StorageAccountClient::new_connection_string(http_client.clone(), &connection_string)?
            .as_storage_client();

    let container: Arc<ContainerClient> = storage_account.as_container_client(&container_name);

    match container.get_properties().execute().await {
        Ok(_) => {}
        Err(_) => {
            // azure_storage::azure_core::errors::UnexpectedHTTPResult is in private crate
            let res = container
                .create()
                .public_access(PublicAccess::Blob)
                .execute()
                .await?;
            println!("{:?}", res);
        }
    }

    // https://docs.rs/chrono/0.4.0/chrono/format/strftime/index.html
    let prefix = Utc::now().format("%Y/%m/%d/%H/%M");

    let filenames: Vec<String> = env::args().skip(1).collect();
    for filename in filenames {
        // Directories which contain characters like '#' contain a %23 in the path when Typora calls us.
        // Therefore, we url-decode the path...
        let filename = urlencoding::decode(&filename).unwrap();

        // Don't like reading everything into memory, on the other hand we're talking JPEG/PNG screenshots,
        // so should be OK
        let mut f = File::open(&filename).await?;
        let mut data = Vec::new();
        f.read_to_end(&mut data).await?;

        let hash = md5::compute(&data[..]);

        let filename_without_extension = Path::new(&filename)
            .file_stem()
            .and_then(OsStr::to_str)
            .unwrap();
        let file_extension_without_dot = Path::new(&filename)
            .extension()
            .and_then(OsStr::to_str)
            .unwrap_or("");

        let base32_encoded_md5 = base32::encode(base32::Alphabet::Crockford, &hash[..]);
        let blob_name = format!(
            "{}/{}----{}.{}",
            prefix, filename_without_extension, base32_encoded_md5, file_extension_without_dot
        );
        let mime_type = get_mimetype_from_filename(&filename);

        let blob = container.as_blob_client(&blob_name);

        blob.put_block_blob(data.clone())
            .content_type(mime_type)
            .hash(&hash.into())
            .execute()
            .await?;

        let mut url = Url::parse(&format!(
            "https://{}.blob.core.windows.net",
            cs.account_name.unwrap()
        ))
        .unwrap();
        url.path_segments_mut().unwrap().push(&container_name);
        for s in blob_name.split('/') {
            url.path_segments_mut().unwrap().push(s);
        }

        // need to tell Typora where the files have been uploaded.
        println!("{}", url);
    }

    Ok(())
}
