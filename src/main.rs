use anyhow::Result;

use clap::{arg, command, Parser};
use hf_hub::{
    api::{tokio::Metadata, RepoInfo},
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

    /// remote repo to upload to
    #[arg(long)]
    repo: Option<String>,

    /// file to upload
    #[arg(long, value_name = "FILE")]
    file_download: Option<String>,

    /// copy the file to dir
    #[arg(long)]
    copy_file: Option<String>,

    /// get info about a remote file
    #[arg(long)]
    file_info: Option<String>,

    /// get info about a remote repo
    #[arg(long)]
    repo_info: bool,

    /// set your local huggingface token
    #[arg(long)]
    set_token: Option<String>,
}

#[tokio::main]
async fn main() {
    let Args {
        file_upload,
        repo,
        file_download,
        copy_file,
        file_info,
        repo_info,
        set_token,
    } = Args::parse();

    match repo {
        Some(repo) => {
            match file_upload {
                Some(filename) => {
                    //let repo = repo.clone().expect("Must specify upload repo");
                    match hf_upload_file(filename, repo.clone()).await {
                        Ok(res) => {
                            println!("{:?}", res)
                        }
                        Err(e) => println!("{:?}", e),
                    };
                }
                None => {}
            }

            match file_download {
                Some(filename) => {
                    // let repo = repo.expect("Must specify upload repo");
                    
                    match hf_download_file(filename, repo.clone(), copy_file).await {
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
                    // let repo = repo_info.clone().expect("Must specify info repo");
                    match hf_get_file_info(filename, repo.clone()).await {
                        Ok(res) => {
                            println!("{:?}", res)
                        }
                        Err(e) => println!("{:?}", e),
                    };
                }
                None => match repo_info {
                    true => match hf_get_repo_info(repo.clone()).await {
                        Ok(res) => {
                            println!("{:?}", res)
                        }
                        Err(e) => println!("{:?}", e),
                    },
                    false => {}
                },
            }
        }
        None => {}
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
    let rel_filename = filename.clone();
    
    let path = Path::new(&filename);
    let data: Vec<u8> = fs::read(path)?; //.await?;
    let filename = path
        .file_name()
        .expect("provide a valid filepath")
        .to_str()
        .expect("provide valid string")
        .to_string();
    println!("{:?} file data length {:?}",filename, data.len());

    let files = [(data, rel_filename)];

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
            "update the files.".to_string().into(),
            false,
        )
        .await?;

    println!("{:?}", res);

    Ok(())
}

async fn hf_download_file(
    filename: String,
    reponame: String,
    copy_to_path: Option<String>,
) -> Result<()> {
    let api = hf_hub::api::tokio::Api::new()?;
    let repo = Repo::model(reponame);
    let api_repo = api.repo(repo);
    let res = api_repo.download(&filename).await;
    println!("{:?}", res);

    match res {
        Ok(p) => {
            println!("Downloaded to your HF .cache folder\n {:?}", p);
            match copy_to_path {
                Some(cp) => {
                    println!("copy to path {:?}",cp);
                    let cp = if cp == ".".to_string() {
                        Path::new(&cp).join(p.file_name().unwrap())
                    } else {
                        Path::new(&cp).to_path_buf()
                    };
                    let res = fs::copy(p, cp);
                    println!("{:?}",res);
                }
                None => {
                    println!("no local copy ");
                }
            }
            return Ok(());
        }
        Err(e) => return Err(e.into()),
    }

    //let path = Path::new(&filename);
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
