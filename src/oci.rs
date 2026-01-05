use flate2::read::GzDecoder;
use reqwest::{blocking::Client, header};
use serde::Deserialize;
use std::{env, error::Error, fs, path::PathBuf};
use tar::Archive;

#[derive(Deserialize)]
struct TokenResponse {
    token: String,
}

#[derive(Deserialize, Debug)]
struct ManifestList {
    manifests: Vec<ManifestEntry>,
}

#[derive(Deserialize, Debug)]
struct ManifestEntry {
    digest: String,
    platform: Platform,
}

#[derive(Deserialize, Debug)]
struct Platform {
    architecture: String,
    os: String,
}

#[derive(Deserialize, Debug)]
struct Manifest {
    layers: Vec<Layer>,
}

#[derive(Deserialize, Debug)]
struct Layer {
    digest: String,
}

const DOCKER_HUB_URL: &str = "https://registry-1.docker.io";
const DOCKER_HUB_AUTH_URL: &str = "https://auth.docker.io/token";
const CACHE_DIR: &str = ".cache/virli/images";

pub fn get_image_path(image_name: &str) -> PathBuf {
    let (name_part, tag) = match image_name.rsplit_once(':') {
        Some((n, t)) => (n, t),
        None => (image_name, "latest"),
    };
    let home = match env::var("HOME") {
        Ok(path) => path,
        Err(_) => "/tmp".to_string(),
    };

    let mut path = PathBuf::from(home);
    path.push(CACHE_DIR);

    for part in name_part.split('/') {
        path.push(part);
    }
    path.push(tag);

    path
}

pub fn fetch_and_extract_image(cache: &PathBuf, image: &str) -> Result<(), Box<dyn Error>> {
    if cache.exists() {
        return Ok(());
    }

    fs::create_dir_all(&cache)?;
    let (image_name, image_tag) = if image.contains(':') {
        let parts: Vec<&str> = image.splitn(2, ':').collect();
        (parts[0], parts[1])
    } else {
        (image, "latest")
    };

    let repository = if !image_name.contains('/') {
        format!("library/{}", image_name)
    } else {
        image_name.to_string()
    };

    // We need to get an authentication token first
    let client = Client::new();
    let auth_url = format!(
        "{}?service=registry.docker.io&scope=repository:{}:pull",
        DOCKER_HUB_AUTH_URL, repository
    );
    let auth_resp: TokenResponse = client.get(&auth_url).send()?.json()?;
    let token = auth_resp.token;

    // Then we can get the manifest or manifest list
    let manifest_url = format!(
        "{}/v2/{}/manifests/{}",
        DOCKER_HUB_URL, repository, image_tag
    );
    let response: serde_json::Value = client
        .get(&manifest_url)
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .header(
            header::ACCEPT,
            "application/vnd.docker.distribution.manifest.v2+json",
        )
        .header(
            header::ACCEPT,
            "application/vnd.docker.distribution.manifest.list.v2+json",
        )
        .send()?
        .json()?;

    let manifest_resp: Manifest = if response.get("manifests").is_some() {
        let manifest_list: ManifestList = serde_json::from_value(response)?;
        let amd64_manifest = manifest_list
            .manifests
            .into_iter()
            .find(|m| m.platform.architecture == "amd64" && m.platform.os == "linux")
            .ok_or("No amd64/linux manifest found")?;

        let manifest_url = format!(
            "{}/v2/{}/manifests/{}",
            DOCKER_HUB_URL, repository, amd64_manifest.digest
        );
        client
            .get(&manifest_url)
            .header(header::AUTHORIZATION, format!("Bearer {}", token))
            .header(
                header::ACCEPT,
                "application/vnd.docker.distribution.manifest.v2+json",
            )
            .send()?
            .json()?
    } else {
        serde_json::from_value(response)?
    };

    for layer in manifest_resp.layers {
        let layer_url = format!(
            "{}/v2/{}/blobs/{}",
            DOCKER_HUB_URL, repository, layer.digest
        );
        let layer_resp = client
            .get(&layer_url)
            .header(header::AUTHORIZATION, format!("Bearer {}", token))
            .send()?;

        let mut gz = GzDecoder::new(layer_resp);
        let mut archive = Archive::new(&mut gz);
        archive.unpack(&cache)?;
    }

    Ok(())
}
