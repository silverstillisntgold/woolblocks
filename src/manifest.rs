use reqwest::blocking::get as http_get;
use serde::Deserialize;
use sha1_smol::Sha1;

#[derive(Debug, Deserialize)]
struct VersionManifestV2 {
    latest: Latest,
    versions: Vec<Version>,
}

#[derive(Debug, Deserialize)]
struct Latest {
    release: String,
    snapshot: String,
}

#[derive(Debug, Deserialize)]
struct Version {
    id: String,
    sha1: String,
    url: String,
}

#[derive(Debug, Deserialize)]
struct PackageManifest {
    downloads: Download,
}

#[derive(Debug, Deserialize)]
struct Download {
    client: ClientData,
}

#[derive(Debug, Deserialize)]
struct ClientData {
    sha1: String,
    size: u64,
    url: String,
}

pub fn xd(version: Option<&str>) {
    /*let response = http_get(crate::MANIFEST_URL).unwrap().bytes().unwrap();
    let manifest: VersionManifestV2 = serde_json::from_slice(&response).unwrap();

    let _test = http_get(crate::MANIFEST_URL).unwrap();
    let _test2 = _test.bytes().unwrap();
    let _wow: VersionManifestV2 = serde_json::from_slice(&_test2).unwrap();
    let _test_sha1 = sha1_smol::Sha1::from(_test2).digest().to_string();
    assert!(_test_sha1 == _wow.versions[0].sha1);*/

    let version = get_version(version);
    let client_data = get_client_data(version);

    //let manifest: PackageManifest = http_get(search.url.as_str()).unwrap().json().unwrap();
}

fn get_version(version: Option<&str>) -> Version {
    let manifest: VersionManifestV2 = http_get(crate::MANIFEST_URL)
        .expect("failed to get version manifest")
        .json()
        .expect("failed to parse JSON");
    // Get version data from the manifest, either using the latest version
    // as provided by said manifest, or from user-provided version.
    let target_version = version.unwrap_or(manifest.latest.release.as_str());
    manifest
        .versions
        .into_iter()
        .find(|v| v.id.as_str().eq(target_version))
        .unwrap()
}

fn get_client_data(version: Version) -> ClientData {
    let package_manifest_bytes = http_get(version.url.as_str()).unwrap().bytes().unwrap();
    let package_manifest_hash = Sha1::from(&package_manifest_bytes).digest().to_string();
    assert!(version.sha1 == package_manifest_hash);

    let package_manifest: PackageManifest =
        serde_json::from_slice(&package_manifest_bytes).unwrap();

    todo!()
}
