use serde::{Deserialize, Serialize};

use crate::error::{FetchError, Result};

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
struct PaperMCBuild {
    pub build: String,
    pub channel: String,
    pub downloads: PaperMCBuildDownloads,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
struct PaperMCBuildDownloads {
    pub application: PaperMCBuildApplication,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
struct PaperMCBuildApplication {
    pub name: String,
    pub sha256: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct PaperMCVersion {
    pub project_name: String,
    pub builds: Vec<PaperMCBuild>,
}

#[derive(Debug, Deserialize, Serialize)]
struct PaperMCProject {
    pub project_name: String,
    pub versions: Vec<String>,
}

pub async fn fetch_papermc_versions(
    project: &str,
    client: &reqwest::Client,
) -> Result<Vec<String>> {
    let res = client
        .get("https://api.papermc.io/v2/projects/".to_owned() + project)
        .send()
        .await?;
    if res.status() == reqwest::StatusCode::NOT_FOUND {
        Err(FetchError::PaperMCProjectNotFound(project.to_owned()))?;
    }
    let project: PaperMCProject = client
        .get("https://api.papermc.io/v2/projects/".to_owned() + project)
        .send()
        .await?
        .json()
        .await?;

    Ok(project.versions)
}

async fn fetch_papermc_builds(
    project: &str,
    version: &str,
    client: &reqwest::Client,
) -> Result<Vec<PaperMCBuild>> {
    let mut target_version: String = version.to_owned();

    if target_version == "latest" {
        let fetched_version = fetch_papermc_versions(project, client).await?;

        if let Some(version) = fetched_version.last().cloned() {
            target_version = version;
        } else {
            Err(FetchError::PaperMCVersionNotFound(
                project.to_owned(),
                "latest".to_owned(),
            ))?;
        }
    }

    let project: PaperMCVersion = client
        .get(
            "https://api.papermc.io/v2/projects/".to_owned()
                + project
                + "/versions/"
                + &target_version
                + "/builds",
        )
        .send()
        .await?
        .json()
        .await?;

    Ok(project.builds)
}

async fn fetch_papermc_build(
    project: &str,
    version: &str,
    build: &str,
    client: &reqwest::Client,
) -> Result<PaperMCBuild> {
    let builds = fetch_papermc_builds(project, version, client).await?;

    if build == "latest" {
        if let Some(build) = builds.last().cloned() {
            return Ok(build);
        }

        Err(FetchError::PaperMCBuildNotFound(
            project.to_owned(),
            version.to_owned(),
            "latest".to_owned(),
        ))?;
    }

    if let Some(found_build) = builds.iter().find(|&b| b.build == build) {
        Ok(found_build.clone())
    } else {
        Err(FetchError::PaperMCBuildNotFound(
            project.to_owned(),
            version.to_owned(),
            build.to_owned(),
        ))?
    }
}

pub async fn download_papermc_build(
    project: &str,
    version: &str,
    build_id: &str,
    client: &reqwest::Client,
) -> Result<reqwest::Response> {
    let build = fetch_papermc_build(project, version, build_id, client).await?;

    let filename = build.downloads.application.name;

    Ok(client
        .get(
            "https://api.papermc.io/v2/projects/".to_owned()
                + project
                + "/versions/"
                + version
                + "/builds/"
                + build_id
                + "/downloads/"
                + &filename,
        )
        .send()
        .await?)
}
