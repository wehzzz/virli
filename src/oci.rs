use flate2::read::GzDecoder;
use reqwest::blocking::Client;
use reqwest::header;
use serde::Deserialize;
use std::error::Error;
use tar::Archive;
use tempfile::TempDir;

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

pub fn fetch_and_extract_image(image: Option<&str>) -> Result<TempDir, Box<dyn Error>> {
    let image = match image {
        Some(img) => img,
        None => return Err("No image specified".into()),
    };

    let (image_name, image_tag) = if image.contains(':') {
        let parts: Vec<&str> = image.split(':').collect();
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
        "https://auth.docker.io/token?service=registry.docker.io&scope=repository:{}:pull",
        repository
    );
    let auth_resp: TokenResponse = client.get(&auth_url).send()?.json()?;
    let token = auth_resp.token;

    // Then we can get the manifest or manifest list
    let manifest_url = format!(
        "https://registry-1.docker.io/v2/{}/manifests/{}",
        repository, image_tag
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
            "https://registry-1.docker.io/v2/{}/manifests/{}",
            repository, amd64_manifest.digest
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

    let temp_dir = TempDir::new()?;
    let rootfs_path = temp_dir.path();

    for layer in manifest_resp.layers {
        let layer_url = format!(
            "https://registry-1.docker.io/v2/{}/blobs/{}",
            repository, layer.digest
        );
        let layer_resp = client
            .get(&layer_url)
            .header(header::AUTHORIZATION, format!("Bearer {}", token))
            .send()?;

        let mut gz = GzDecoder::new(layer_resp);
        let mut archive = Archive::new(&mut gz);
        archive.unpack(rootfs_path)?;
    }

    Ok(temp_dir)
}
