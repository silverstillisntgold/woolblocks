use reqwest::blocking::get as http_get;
use serde::Deserialize;
use sha1_smol::Sha1;

#[derive(Deserialize)]
struct VersionManifestV2 {
    latest: Latest,
    versions: Vec<Version>,
}

#[derive(Deserialize)]
struct Latest {
    release: String,
    snapshot: String,
}

#[derive(Deserialize)]
struct Version {
    id: String,
    sha1: String,
    url: String,
}

#[derive(Deserialize)]
struct PackageManifest {
    downloads: Download,
}

#[derive(Deserialize)]
struct Download {
    client: ClientData,
}

#[derive(Deserialize)]
struct ClientData {
    sha1: String,
    size: u64,
    url: String,
}

/// Returns the raw bytes the client jar for the passed `version_id`,
/// or the latest version if `None` is passed.
pub fn get_client_jar_as_bytes(version_id: Option<&str>) -> Box<[u8]> {
    let version = get_version(version_id);
    let client_data = get_client_data(version);
    get_raw_client_bytes(client_data)
}

fn get_version(version_id: Option<&str>) -> Version {
    let manifest: VersionManifestV2 = http_get(crate::MANIFEST_URL)
        .expect("failed to get version manifest")
        .json()
        .expect("failed to parse JSON of version manifest");
    // Get version data from the manifest, either using the latest version
    // as provided by said manifest, or from user-provided version.
    let target_version = version_id.unwrap_or(manifest.latest.release.as_str());
    manifest
        .versions
        .into_iter()
        .find(|v| v.id.as_str().eq(target_version))
        .expect("this should be unreachable")
}

fn get_client_data(version: Version) -> ClientData {
    let package_manifest_bytes = http_get(version.url.as_str()).unwrap().bytes().unwrap();
    let package_manifest_hash = Sha1::from(&package_manifest_bytes).digest().to_string();
    assert!(
        version.sha1 == package_manifest_hash,
        "sha1 validation of package manifest for version \"{}\" failed",
        version.id.as_str()
    );
    let package_manifest: PackageManifest = serde_json::from_slice(&package_manifest_bytes)
        .expect("failed to parse JSON of package manifest");
    package_manifest.downloads.client
}

fn get_raw_client_bytes(client_data: ClientData) -> Box<[u8]> {
    // Sanity check
    assert!(client_data.url.ends_with("client.jar"), "bruh wtf");
    let client_bytes = http_get(client_data.url.as_str())
        .expect("failed to get client jar")
        .bytes()
        .unwrap();
    // No need to bother computing/comparing hashes if sizes are mismatched.
    assert!(
        client_data.size == client_bytes.len() as u64,
        "incorrect file size for client jar"
    );
    let client_hash = Sha1::from(&client_bytes).digest().to_string();
    assert!(
        client_data.sha1 == client_hash,
        "sha1 validation of client jar failed"
    );
    // This is a copy operation.
    // It seems like it's not currently possible to do this in
    // a way that moves the underlying data instead of copying it,
    // but since the client jar is relatively small it shouldn't
    // make all that much of a difference in performance.
    client_bytes.to_vec().into_boxed_slice()
}
