A simple cli for interfacing with huggingface hub.

**This is Experimental software and is subject to change.**


To install build and copy the executable to your bin path

```
cargo build --release
./install-linux.sh
```


then run hf-cli anywhere to upload files or get info about remote repositories
```
$ hf-cli --help
Usage: hf-cli [OPTIONS]

Options:
      --file-upload <FILE>         file to upload
      --repo-upload <REPO_UPLOAD>  remote repo to upload to
      --file-info <FILE_INFO>      get info about a remote file
      --repo-info <REPO_INFO>      get info about a remote repo
      --set-token <SET_TOKEN>      set your local huggingface token
  -h, --help                       Print help
  -V, --version                    Print version
  ```
