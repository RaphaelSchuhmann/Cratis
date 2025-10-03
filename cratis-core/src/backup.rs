use crate::error::{display_msg, CratisErrorLevel, CratisResult};
use crate::utils::{is_path_file, get_files_in_directory, load_file};
use crate::config::get_config_cli;
use reqwest::{Client};
use std::fs::File;
use std::path::PathBuf;
use tokio::fs::File as TokioFile;
use tokio_util::io::ReaderStream;

pub async fn backup() -> reqwest::StatusCode {
    let watch_dirs = &get_config_cli().backup.watch_directories;

    let mut files_to_load: Vec<PathBuf> = Vec::new();

    for dir in watch_dirs {
        if is_path_file(dir) {
            files_to_load.push(PathBuf::from(dir));
        } else {
            let files: CratisResult<Vec<PathBuf>> = get_files_in_directory(dir);
            match files {
                Ok(files) => {
                    files_to_load.extend(files);
                }
                Err(e) => {
                    display_msg(Some(&e), CratisErrorLevel::Warning, None)
                }
            }
        }
    }

    let mut loaded_files: Vec<(File, String, String)> = Vec::new();

    for file in files_to_load {
        let loaded_file = load_file(file);
        match loaded_file {
            Ok(file) => {
                loaded_files.push((file.0, file.1, file.2.unwrap()));
            }
            Err(e) => {
                display_msg(Some(&e), CratisErrorLevel::Warning, None)
            }
        }
    }

    // Put loaded files into request body
    let mut form = reqwest::multipart::Form::new();

    for (std_file, file_name, file_path) in loaded_files {
        let tokio_file: TokioFile = TokioFile::from_std(std_file);
        let file_body_stream = ReaderStream::new(tokio_file);
        let body = reqwest::Body::wrap_stream(file_body_stream);
        let file_part = reqwest::multipart::Part::stream(body).file_name(file_name).mime_str("application/octet-stream").expect("Unable to send files");

        form = form.part("files", file_part);
        form = form.text("paths", file_path);
    }

    let client = Client::new();
    let config = get_config_cli();

    // Send request
    let response = client.post(format!("{}/backup", config.server.address))
        .bearer_auth(config.server.auth_token.clone())
        .multipart(form)
        .send()
        .await
        .expect("Invalid request");

    let status: reqwest::StatusCode = response.status().into();
    status
}