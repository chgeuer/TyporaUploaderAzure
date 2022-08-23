use azure_core::prelude::*;
use azure_storage::blob::prelude::*;
use azure_storage::core::prelude::*;
use chrono::Utc;
use reqwest::header::CONTENT_TYPE;
use reqwest::Response;
use std::env;
use std::error::Error;
use std::ffi::OsStr;
use std::path::Path;
use std::sync::Arc;
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use url::Url;

const CONNECTIONSTR_ENVVAR_NAME: &str = "TYPORA_IMAGE_UPLOAD_AZURE_CONNECTION";
const CONTAINER_ENVVAR_NAME: &str = "TYPORA_IMAGE_UPLOAD_AZURE_CONTAINER";
const UPLOAD_VANITY_HOSTNAME: &str = "TYPORA_IMAGE_UPLOAD_VANITY_HOST";

fn get_mimetype_from_extension(extension: &str) -> &str {
    // could also use mime_guess = "2.0.3"
    match extension {
        "jpg" => "image/jpeg",
        "jpeg" => "image/jpeg",
        "png" => "image/png",
        "svg" => "image/svg+xml",
        "mp4" => "video/mp4",
        "pdf" => "application/pdf",
        _ => "application/octet-stream",
    }
}

struct UploadData {
    blob_base_name: String,
    extension: String,
    mime_type: String,
    source: String,
    bytes: Vec<u8>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let connection_string = std::env::var(CONNECTIONSTR_ENVVAR_NAME)
        .expect("Set env variable {CONNECTIONSTR_ENVVAR_NAME} first!");

    // let container_name: String = "typoraimages".to_owned();
    let container_name = std::env::var(CONTAINER_ENVVAR_NAME)
        .expect("Set env variable {CONTAINER_ENVVAR_NAME} first!");

    let cs = azure_storage::ConnectionString::new(&connection_string).unwrap();

    let http_client: Arc<Box<dyn HttpClient>> = Arc::new(Box::new(reqwest::Client::new()));

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
    let date = Utc::now().format("%Y/%m/%d/%H/%M").to_string();

    // https://georgik.rocks/how-to-download-binary-file-in-rust-by-reqwest/

    let filenames: Vec<String> = env::args().skip(1).collect();
    for filename_or_url in filenames {
        let upload_data_option =
            if filename_or_url.starts_with("http://") || filename_or_url.starts_with("https://") {
                let url = filename_or_url;
                let url = Url::parse(&url).unwrap();
                let filename = url.path().split('/').last().unwrap();

                let blob_base_name = Path::new(&filename)
                    .file_stem()
                    .and_then(OsStr::to_str)
                    .unwrap()
                    .to_string();
                let extension = Path::new(&filename)
                    .extension()
                    .and_then(OsStr::to_str)
                    .unwrap_or_default()
                    .to_string();

                let content: Response = reqwest::get(url.clone()).await?;
                if content.status().is_success() {
                    let mime_type = match content.headers().get(CONTENT_TYPE) {
                        Some(content_type) => match content_type.to_str() {
                            Ok(mimetype) => mimetype,
                            _ => "application/binary",
                        },
                        None => "application/binary",
                    }
                    .to_string();

                    let bytes = content.bytes().await?.to_vec();

                    Some(UploadData {
                        blob_base_name,
                        extension,
                        mime_type,
                        source: url.to_string(),
                        bytes,
                    })
                } else {
                    None
                }
            } else {
                let filename = filename_or_url;
                let path: &Path = Path::new(&filename);

                let blob_base_name = path
                    .file_stem()
                    .and_then(OsStr::to_str)
                    .unwrap()
                    .to_string();
                let extension = path.extension().and_then(OsStr::to_str).unwrap();

                let mime_type = get_mimetype_from_extension(extension).to_string();

                let mut f = File::open(&filename).await?;
                let mut bytes = Vec::new();
                f.read_to_end(&mut bytes).await?;

                Some(UploadData {
                    blob_base_name,
                    extension: extension.to_string(),
                    mime_type,
                    source: filename,
                    bytes,
                })
            };

        match upload_data_option {
            None => {
                println!("");
            }
            Some(UploadData {
                blob_base_name,
                extension,
                mime_type,
                source,
                bytes,
            }) => {
                let hash = md5::compute(&bytes[..]);
                let base32_encoded_md5: String =
                    base32::encode(base32::Alphabet::Crockford, &hash[..]);

                let blob_name = if extension.is_empty() {
                    format!("{date}/{blob_base_name}----{base32_encoded_md5}")
                } else {
                    format!("{date}/{blob_base_name}----{base32_encoded_md5}.{extension}")
                };

                let mut metadata = Metadata::new();
                metadata.as_mut().insert("source".into(), source.into());

                container
                    .as_blob_client(&blob_name)
                    .put_block_blob(bytes.clone())
                    .content_type(&mime_type[..])
                    .metadata(&metadata)
                    //.content_disposition(&format!("attachment; filename={}.{}", filename_without_extension, file_extension_without_dot)[..])
                    //.content_language("en-us")
                    .hash(&hash.into())
                    .execute()
                    .await?;

                let hostname = match std::env::var(UPLOAD_VANITY_HOSTNAME) {
                    Ok(h) => format!("http://{}", h),
                    Err(_) => format!("https://{}.blob.core.windows.net", cs.account_name.unwrap()),
                };

                let mut url: Url = Url::parse(&hostname).unwrap();
                url.path_segments_mut().unwrap().push(&container_name);
                for s in blob_name.split('/') {
                    url.path_segments_mut().unwrap().push(s);
                }

                // need to tell Typora where the files have been uploaded.
                // println!("{blob_name} : {mime_type} : {len} bytes");

                println!("{}", url);
            }
        }
    }
    Ok(())
}
