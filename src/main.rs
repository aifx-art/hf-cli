use anyhow::Result;

use clap::{arg, command, Parser};
use hf_hub::{
    api::{
        tokio::Metadata,
        RepoInfo,
    },
    Repo,
};
use tokio::{fs::File, io::AsyncWriteExt};

use std::{
    fs::{self, ReadDir},
    path::Path,
};
//use tokio::fs::{self, ReadDir};
//use std::path::Path;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// file to upload
    #[arg(long, value_name = "FILE")]
    file_upload: Option<String>,
    #[arg(long)]
    repo_upload: Option<String>,

    #[arg(long)]
    file_info: Option<String>,

    #[arg(long)]
    repo_info: Option<String>,

    #[arg(long)]
    set_token: Option<String>,
}

#[tokio::main]
async fn main() {
    
    let Args {
        file_upload,
        repo_upload,
        file_info,
        repo_info,
        set_token,
    } = Args::parse();

    match file_upload {
        Some(filename) => {
            let repo = repo_upload.expect("Must specify upload repo");
            match hf_upload_file(filename, repo).await {
                Ok(res) => {
                    println!("{:?}", res)
                }
                Err(e) => println!("{:?}", e),
            };
        }
        None => {}
    }

    match file_info {
        Some(filename) => {
            let repo = repo_info.clone().expect("Must specify info repo");
            match hf_get_file_info(filename, repo).await {
                Ok(res) => {
                    println!("{:?}", res)
                }
                Err(e) => println!("{:?}", e),
            };
        }
        None => match repo_info {
            Some(reponame) => match hf_get_repo_info(reponame).await {
                Ok(res) => {
                    println!("{:?}", res)
                }
                Err(e) => println!("{:?}", e),
            },
            None => {}
        },
    }

    match set_token {
        Some(token) => {
            let res = set_huggingface_token(token).await;
            println!("{:?}", res);
        }
        None => {}
    }

    println!("done");
}

async fn hf_get_file_info(filename: String, reponame: String) -> Result<Metadata> {
    println!("get file info for {} / {}", reponame, filename,);
    let api = hf_hub::api::tokio::Api::new()?;
    let repo = Repo::model(reponame);
    let url = api.repo(repo).file_url(&filename);
    let md: Metadata = api.metadata(&url).await?;

    Ok(md)
}

async fn hf_get_repo_info(reponame: String) -> Result<RepoInfo> {
    println!("get repo info for {}", reponame);
    let api = hf_hub::api::tokio::Api::new()?;
    let repo = Repo::model(reponame);
    let repo_info: RepoInfo = api.repo(repo).info().await?;

    Ok(repo_info)
}

async fn hf_upload_file(filename: String, reponame: String) -> Result<()> {
    println!("upload file {} to {}", filename, reponame);
    let path = Path::new(&filename);
    let data: Vec<u8> = fs::read(path)?; //.await?;
    let filename = path
        .file_name()
        .expect("provide a valid filepath")
        .to_str()
        .expect("provide valid string")
        .to_string();
    println!("file data length {:?}", data.len());

    let files = [(data, filename)];

    //let api = ApiBuilder::new().build()
    let api = hf_hub::api::tokio::Api::new()?;
    let repo = Repo::model(reponame);
    let api_repo = api.repo(repo);

    let files = files
        .into_iter()
        .map(|(data, path)| (data.into(), path.into()))
        .collect();

    println!("files converted {:?}", files);

    println!("uploading files...");
    let res = api_repo
        .upload_files(
            files,
            None,
            "update multiple files!".to_string().into(),
            false,
        )
        .await?;

    println!("{:?}", res);

    Ok(())
}

pub async fn set_huggingface_token(token: String) -> Result<(), String> {
    let cache = hf_hub::Cache::default();

    let token_path = cache.token_path();
    println!("token path {:?}", token_path);

    let cache_path = token_path.parent().unwrap();
    let ls = read_create_dir(cache_path.to_str().unwrap()).await;
    println!("read dir {:?}", ls);

    match File::create(token_path).await {
        Ok(mut file) => {
            match file.write_all(token.as_bytes()).await {
                Ok(_) => return Ok(()),
                Err(e) => return Err(e.to_string()),
            };
        }
        Err(e) => {
            println!("Error creating file {:?}", e);
            return Err(e.to_string());
        }
    };
}

pub async fn read_create_dir(path: &str) -> ReadDir {
    let dir;
    match fs::read_dir(path) {
        Ok(p) => {
            dir = p;
        }

        Err(_) => {
            println!("Creating up {}", path);
            fs::create_dir(path).unwrap();
            dir = fs::read_dir(path).unwrap();
        }
    };
    dir
}
