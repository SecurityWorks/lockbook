use crate::model::file_like::FileLike;
use crate::model::signed_file::SignedFile;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct ServerFile {
    pub file: SignedFile,
    pub version: u64,
}

pub trait IntoServerFile {
    fn add_time(self, version: u64) -> ServerFile;
}

impl IntoServerFile for SignedFile {
    fn add_time(self, version: u64) -> ServerFile {
        ServerFile { file: self, version }
    }
}

impl Display for ServerFile {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display())
    }
}
