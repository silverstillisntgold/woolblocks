use crate::{MANIFEST_URL, Version};
use serde::Deserialize;
use sha1_smol::Sha1;
use ureq::get as https_get;

#[derive(Deserialize)]
struct VersionManifestV2 {
    latest: LatestData,
    versions: Vec<VersionData>,
}

#[derive(Deserialize)]
struct LatestData {
    release: String,
    snapshot: String,
}

#[derive(Deserialize)]
struct VersionData {
    id: String,
    sha1: String,
    url: String,
}

#[derive(Deserialize)]
struct PackageManifest {
    downloads: DownloadData,
}

#[derive(Deserialize)]
struct DownloadData {
    client: ClientData,
}

#[derive(Deserialize)]
struct ClientData {
    sha1: String,
    url: String,
}

/// Return the raw bytes of the client jar for the passed `version_id`.
pub fn get_client_jar_as_bytes(version_id: Version) -> Box<[u8]> {
    let version = get_version(version_id);
    let client_data = get_client_data(version);
    get_raw_client_bytes(client_data)
}

fn get_version(version_id: Version) -> VersionData {
    let version_manifest = https_get(MANIFEST_URL)
        .call()
        .unwrap()
        .into_body()
        .read_json::<VersionManifestV2>()
        .unwrap();
    // Invalid `Custom` variant will cause a panic.
    let target_version = match version_id {
        Version::Custom(version) => version,
        Version::Release => version_manifest.latest.release.as_str(),
        Version::Snapshot => version_manifest.latest.snapshot.as_str(),
    };
    version_manifest
        .versions
        .into_iter()
        .find(|v| v.id.as_str().eq(target_version))
        .expect("the `version_id` provided should be a valid minecraft version")
}

fn get_client_data(version: VersionData) -> ClientData {
    let package_manifest_bytes = https_get(&version.url)
        .call()
        .unwrap()
        .into_body()
        .into_with_config()
        .limit(u32::MAX as u64)
        .read_to_vec()
        .unwrap();
    let package_manifest_hash = Sha1::from(&package_manifest_bytes).digest().to_string();
    assert!(
        version.sha1 == package_manifest_hash,
        "sha1 validation of package manifest failed"
    );
    serde_json::from_slice::<PackageManifest>(&package_manifest_bytes)
        .unwrap()
        .downloads
        .client
}

fn get_raw_client_bytes(client_data: ClientData) -> Box<[u8]> {
    let client_bytes = https_get(&client_data.url)
        .call()
        .unwrap()
        .into_body()
        .into_with_config()
        .limit(u32::MAX as u64)
        .read_to_vec()
        .unwrap();
    let client_hash = Sha1::from(&client_bytes).digest().to_string();
    assert_eq!(
        client_data.sha1, client_hash,
        "sha1 validation of client jar failed"
    );
    client_bytes.into_boxed_slice()
}
