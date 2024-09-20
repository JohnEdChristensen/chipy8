use std::{
    fs,
    path::{Path, PathBuf},
};
#[derive(Clone, PartialEq)]
pub struct Rom {
    path: PathBuf,
    pub contents: Vec<u8>,
}
impl Rom {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, std::io::Error> {
        let path_buf = path.as_ref().to_path_buf();
        let contents = fs::read(&path_buf).unwrap();
        Ok(Self {
            path: path_buf,
            contents,
        })
    }
    pub fn name(&self) -> &str {
        self.path.file_stem().unwrap().to_str().unwrap()
    }
}
