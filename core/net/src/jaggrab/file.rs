use std::convert::TryFrom;

use super::JaggrabError;

#[derive(Debug)]
pub enum JaggrabFile {
    Title = 1,
    Config = 2,
    Interface = 3,
    Media = 4,
    VersionList = 5,
    Textures = 6,
    WordEnc = 7,
    Sounds = 8,
}

impl TryFrom<String> for JaggrabFile {
    type Error = JaggrabError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_ref() {
            "title" => Ok(JaggrabFile::Title),
            "config" => Ok(JaggrabFile::Config),
            "interface" => Ok(JaggrabFile::Interface),
            "media" => Ok(JaggrabFile::Media),
            "versionlist" => Ok(JaggrabFile::VersionList),
            "textures" => Ok(JaggrabFile::Textures),
            "wordenc" => Ok(JaggrabFile::WordEnc),
            "sounds" => Ok(JaggrabFile::Sounds),
            _ => Err(JaggrabError::FileNotFound(value)),
        }
    }
}
