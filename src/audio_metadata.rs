use lofty::prelude::*;
use serde::Serialize;
use std::path::Path;
use std::fs::read_dir;
use lofty::probe::Probe;

#[derive(Debug, Clone, Serialize)]
pub struct Track {
    pub file: String,
    pub title: String,
    pub author: String,
    pub src: String,
}

pub fn get_playlist(audio_path: Option<&String>) -> Vec<Track> {
    let mut playlist = Vec::new();
    let files_list = read_dir(audio_path.unwrap_or(&"assets/soundtracks".to_string())).unwrap();
    let mut id_filename = 0;
    for file in files_list {
        if let Ok(file) = file {

            if file.metadata().unwrap().is_file() {
                id_filename += 1;
                let path_str = if audio_path.is_some() {
                    format!("{}/{}.mp3", audio_path.unwrap(), id_filename)
                } else {
                    format!("assets/soundtracks/{}.mp3", id_filename)
                };
                let path = Path::new(&path_str);

                if path.exists() {
                    let mut title = format!("Log_Track_{}", id_filename);
                    let mut author = "Unknown_Source".to_string();

                    if let Ok(tagged_file) = Probe::open(path).and_then(|p| p.read()) {
                        if let Some(tag) = tagged_file.primary_tag().or_else(|| tagged_file.first_tag()) {
                            title = tag.title().map(|s| s.to_string()).unwrap_or(title);
                            author = tag.artist().map(|s| s.to_string()).unwrap_or(author);
                        }
                    }

                    playlist.push(Track {
                        file: format!("{}.mp3", id_filename),
                        title,
                        author,
                        src: format!("/{}", path_str)
                    });
                }
            }
        }
    }

    playlist
}