use crate::{MANIFEST_URL, MismatchError, Version, WoolError};
use serde::Deserialize;
use sha1_smol::{Digest, Sha1};
use std::io::Read;

#[derive(Deserialize)]
struct VersionManifestV2 {
    latest: LatestData,
    versions: Box<[VersionData]>,
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
    downloads: DownloadData,
    id: String,
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

/// Returns the raw bytes of the client jar corresponding to the passed `version`.
#[inline(never)]
pub fn get_client_jar_bytes(version: Version) -> Result<Box<[u8]>, WoolError> {
    let version_data = get_version(version)?;
    let client_data = get_client_data(version_data)?;
    get_raw_client_bytes(client_data)
}

fn get_version(version: Version) -> Result<VersionData, WoolError> {
    let version_manifest_bytes = get_url_body(MANIFEST_URL)?;
    let version_manifest = serde_json::from_slice::<VersionManifestV2>(&version_manifest_bytes)?;

    // Cache this value to avoid having a `match` statement in our `find` loop.
    let target_version_id = match version {
        Version::Exact(id) => id,
        Version::Release => &version_manifest.latest.release,
        Version::Snapshot => &version_manifest.latest.snapshot,
    };

    // Not bothering to use `rayon` here because there aren't enough Minecraft
    // versions to make a tangible difference in search speed.
    // It's also most likely that packs will be generated for newer versions, which
    // are at the front of `versions` and will be found [almost] immediately.
    version_manifest
        .versions
        .into_iter()
        .find(|version_data| version_data.id.eq(target_version_id))
        .ok_or(WoolError::InvalidVersion)
}

fn get_client_data(version_data: VersionData) -> Result<ClientData, WoolError> {
    let package_manifest_bytes = get_url_body(&version_data.url)?;

    let package_manifest_sha1 = Sha1::from(&package_manifest_bytes).digest();
    if version_data.sha1 != package_manifest_sha1 {
        return Err(MismatchError::VersionManifestSha1.into());
    }

    let package_manifest = serde_json::from_slice::<PackageManifest>(&package_manifest_bytes)?;
    if version_data.id != package_manifest.id {
        return Err(MismatchError::VersionId.into());
    }

    Ok(package_manifest.downloads.client)
}

fn get_raw_client_bytes(client_data: ClientData) -> Result<Box<[u8]>, WoolError> {
    let client_bytes = get_url_body(&client_data.url)?;

    let client_data_sha1 = Sha1::from(&client_bytes).digest();
    if client_data.sha1 != client_data_sha1 {
        return Err(MismatchError::ClientJarSha1.into());
    }

    Ok(client_bytes)
}

/// The [`ureq`] crate doesn't do any internal pre-allocation when fetching HTTP bodies
/// (idk why not maybe they're retarded?), so we need to do it ourselves.
#[inline(never)]
fn get_url_body(url: &str) -> Result<Box<[u8]>, WoolError> {
    // Effectively unlimited for the expected JAR size (<50MB).
    const LIMIT: u64 = 1 << 29;

    let response = ureq::get(url).call()?;

    // The buffer will attempt to size itself according to `Content-Length`, falling back to
    // using the upper limit if that field isn't provided by the body. This ensures the buffer
    // will never need to be resized while reading the body.
    let capacity = response
        .body()
        .content_length()
        .map(|len| len.min(LIMIT))
        .unwrap_or(LIMIT) as usize;
    let mut buf = Vec::with_capacity(capacity);

    response
        .into_body()
        .into_with_config()
        .limit(LIMIT)
        .reader()
        .read_to_end(&mut buf)?;

    Ok(buf.into_boxed_slice())
}
