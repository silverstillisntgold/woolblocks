use crate::MANIFEST_URL;
use reqwest::blocking::get as https_get;
use serde::Deserialize;
use sha1_smol::Sha1;

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
    size: u64,
    url: String,
}

pub enum Version<'a> {
    Custom(&'a str),
    Release,
    Snapshot,
}

/// Returns the raw bytes of the client jar for the passed `version_id`,
/// or for the latest stable version if `None` is passed.
pub fn get_client_jar_as_bytes(version_id: Version) -> Box<[u8]> {
    let version = get_version(version_id);
    let client_data = get_client_data(version);
    get_raw_client_bytes(client_data)
}

fn get_version(version_id: Version) -> VersionData {
    let version_manifest = https_get(MANIFEST_URL)
        .unwrap()
        .json::<VersionManifestV2>()
        .unwrap();
    let target_version = match version_id {
        Version::Custom(version) => version,
        Version::Release => version_manifest.latest.release.as_str(),
        Version::Snapshot => version_manifest.latest.snapshot.as_str(),
    };
    version_manifest
        .versions
        .into_iter()
        .find(|v| v.id.as_str().eq(target_version))
        // Should only be reachable when using incorrect `Version::Custom` from user.
        .expect(&format!(
            "provided 'target_version' {} should be a valid Minecraft version",
            target_version
        ))
}

fn get_client_data(version: VersionData) -> ClientData {
    let package_manifest_bytes = https_get(&version.url).unwrap().bytes().unwrap();
    let package_manifest_hash = Sha1::from(&package_manifest_bytes).digest().to_string();
    assert!(
        version.sha1 == package_manifest_hash,
        "sha1 validation of package manifest for version \"{}\" failed",
        &version.id
    );
    serde_json::from_slice::<PackageManifest>(&package_manifest_bytes)
        .unwrap()
        .downloads
        .client
}

fn get_raw_client_bytes(client_data: ClientData) -> Box<[u8]> {
    let client_bytes = https_get(&client_data.url).unwrap().bytes().unwrap();
    // Don't bother computing/comparing hashes if sizes are mismatched.
    assert_eq!(
        client_data.size,
        client_bytes.len() as u64,
        "incorrect file size for client jar"
    );
    let client_hash = Sha1::from(&client_bytes).digest().to_string();
    assert_eq!(
        client_data.sha1, client_hash,
        "sha1 validation of client jar failed"
    );
    // It seems like it's not currently possible to do this in
    // a way that moves the underlying data instead of copying it.
    client_bytes.to_vec().into_boxed_slice()
}
