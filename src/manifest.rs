use crate::{MANIFEST_URL, Version, WoolError};
use serde::Deserialize;
use sha1_smol::{Digest, Sha1};
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
    sha1: Digest,
    url: String,
}

#[derive(Deserialize)]
struct PackageManifest {
    id: String,
    downloads: DownloadData,
}

#[derive(Deserialize)]
struct DownloadData {
    client: ClientData,
}

#[derive(Deserialize)]
struct ClientData {
    sha1: Digest,
    url: String,
}

/// Return the raw bytes of the client jar for the passed `version_id`.
#[inline(never)]
pub fn get_client_jar_as_bytes(version_id: Version) -> Result<Box<[u8]>, WoolError> {
    let version = get_version(version_id)?;
    let client_data = get_client_data(version)?;
    get_raw_client_bytes(client_data)
}

fn get_version(version_id: Version) -> Result<VersionData, WoolError> {
    let version_manifest = https_get(MANIFEST_URL)
        .call()?
        .into_body()
        .read_json::<VersionManifestV2>()?;

    // Cache this here so we avoid having a `match` statement in our `find` loop.
    let target_version = match version_id {
        Version::Custom(version) => version,
        Version::Release => &version_manifest.latest.release,
        Version::Snapshot => &version_manifest.latest.snapshot,
    };

    // Not bothering to use rayon because there aren't enough Minecraft
    // versions to make a tangible difference in search speed.
    // It's also most likely that packs will be generated for newer versions, which
    // are at the front of `versions` and will be found [almost] immediately.
    version_manifest
        .versions
        .into_iter()
        .find(|version| version.id.eq(target_version))
        .ok_or(WoolError::InvalidVersion)
}

fn get_client_data(version: VersionData) -> Result<ClientData, WoolError> {
    let package_manifest_bytes = https_get(&version.url).call()?.into_body().read_to_vec()?;

    let package_manifest_sha1 = Sha1::from(&package_manifest_bytes).digest();
    if version.sha1 != package_manifest_sha1 {
        return Err(WoolError::MismatchSha1Manifest);
    }

    let package_manifest = serde_json::from_slice::<PackageManifest>(&package_manifest_bytes)?;
    if version.id != package_manifest.id {
        return Err(WoolError::MismatchVersion);
    }

    Ok(package_manifest.downloads.client)
}

fn get_raw_client_bytes(client_data: ClientData) -> Result<Box<[u8]>, WoolError> {
    let client_bytes = https_get(&client_data.url)
        .call()?
        .into_body()
        .into_with_config()
        .limit(i32::MAX as u64)
        .read_to_vec()?;

    let client_sha1 = Sha1::from(&client_bytes).digest();
    if client_data.sha1 != client_sha1 {
        return Err(WoolError::MismatchSha1Data);
    }

    Ok(client_bytes.into_boxed_slice())
}
