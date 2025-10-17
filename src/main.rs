use anyhow::Result;

use clap::{arg, command, Parser};
use hf_hub::{
    api::{tokio::Metadata, RepoInfo},
    Repo, RepoType,
};
use tokio::{fs::File, io::AsyncWriteExt};

use std::{
    fs::{self, ReadDir},
    path::{Path, PathBuf},
};
//use tokio::fs::{self, ReadDir};
//use std::path::Path;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// file to upload
    #[arg(long, value_name = "FILE")]
    upload_file: Option<String>,

    /// file to upload
    #[arg(long, value_name = "FILE")]
    upload_repo: bool,

    /// remote repo to interact with
    #[arg(long)]
    repo: Option<String>,

    #[arg(long)]
    repotype: Option<RepoType>,

    /// file to upload
    #[arg(long, value_name = "FILE")]
    download_file: Option<String>,

    /// copy the file/s to dir
    #[arg(long)]
    copy_file: Option<String>,

    /// get info about a remote file
    #[arg(long)]
    file_info: Option<String>,

    /// get info about a remote repo
    #[arg(long)]
    repo_info: bool,

    /// download the entire repo
    #[arg(long)]
    download_repo: bool,

    /// set your local huggingface token
    #[arg(long)]
    set_token: Option<String>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let Args {
        upload_file,
        upload_repo,
        repo,
        download_file,
        copy_file,
        file_info,
        repo_info,
        download_repo,
        set_token,
        repotype,
    } = Args::parse();

    match repo {
        Some(repo) => {
            match upload_file {
                Some(filename) => {
                    //let repo = repo.clone().expect("Must specify upload repo");

                    match hf_upload_file(filename, repo.clone(), repotype).await {
                        Ok(res) => {
                            println!("{:?}", res)
                        }
                        Err(e) => println!("{:?}", e),
                    };
                }
                None => {}
            }

            match upload_repo {
                true => {
                    let files = get_all_paths(".")?;
                    //let repo = repo.clone().expect("Must specify upload repo");
                    for file in files {
                        match hf_upload_file(file, repo.clone(), repotype).await {
                            Ok(res) => {
                                println!("{:?}", res)
                            }
                            Err(e) => println!("{:?}", e),
                        };
                    }
                }
                false => {}
            }

            match download_file {
                Some(filename) => {
                    // let repo = repo.expect("Must specify upload repo");

                    match hf_download_file(filename, repo.clone(), copy_file.clone()).await {
                        Ok(res) => {
                            println!("{:?}", res)
                        }
                        Err(e) => println!("{:?}", e),
                    };
                }
                None => {}
            }
            match download_repo {
                true => {
                    //TODO get all the files and downlaod them
                    let info = hf_get_repo_info(repo.clone()).await?;
                    let mut files: Vec<String> = vec![];
                    for s in info.siblings.into_iter() {
                        files.push(s.rfilename);
                    }
                    println!("files {:?}", files);
                    for f in files {
                        match hf_download_file(f, repo.clone(), copy_file.clone()).await {
                            Ok(res) => {
                                println!("{:?}", res)
                            }
                            Err(e) => println!("{:?}", e),
                        }
                    }
                }
                false => {}
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
    Ok(())
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

//--repo-type=dataset
async fn hf_upload_file(
    filename: String,
    reponame: String,
    repotype: Option<RepoType>,
) -> Result<()> {
    let repotype = repotype.unwrap_or(RepoType::Model);

    println!("upload file {} to {} of {:?}", filename, reponame, repotype);
    let rel_filename = filename.clone();

    let path = Path::new(&filename);
    let data: Vec<u8> = fs::read(path)?; //.await?;
    let filename = path
        .file_name()
        .expect("provide a valid filepath")
        .to_str()
        .expect("provide valid string")
        .to_string();
    println!("{:?} file data length {:?}", filename, data.len());

    let files = [(data, rel_filename)];

    //let api = ApiBuilder::new().build()
    let api = hf_hub::api::tokio::Api::new()?;
    //let repo = Repo::model(reponame);
    let repo = Repo::new(reponame, repotype);

    let api_repo = api.repo(repo);

    let files = files
        .into_iter()
        .map(|(data, path)| (data.into(), path.into()))
        .collect();

    println!("files converted {:?}", files);

    println!("uploading files...");
    let res = api_repo
        .upload_files(files, None, "update the files.".to_string().into(), false)
        .await?;

    println!("{:?}", res);
    println!("Finished uploading files.");
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
                    println!("copy to path {:?}", cp);
                    let cp = if cp == ".".to_string() {
                        Path::new(&cp).join(p.file_name().unwrap())
                    } else {
                        Path::new(&cp).to_path_buf()
                    };
                    let res = fs::copy(p, cp);
                    println!("{:?}", res);
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

fn get_all_paths(dir: &str) -> Result<Vec<String>> {
    let base = Path::new(dir);
    let mut paths = Vec::new();
    collect_paths(base, base, &mut paths)?;
    Ok(paths)
}

fn collect_paths(base: &Path, current: &Path, paths: &mut Vec<String>) -> Result<()> {
    for entry in fs::read_dir(current)? {
        let entry = entry?;
        let path = entry.path();
        
        // Get path relative to base directory
        if let Ok(rel_path) = path.strip_prefix(base) {
            paths.push(rel_path.to_string_lossy().to_string());
        }
        
        if path.is_dir() {
            collect_paths(base, &path, paths)?;
        }
    }
    
    Ok(())
}