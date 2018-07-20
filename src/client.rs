use std::path::Path;

use error::*;

pub struct Client {
}

impl Client {
    pub fn with_path<P: AsRef<Path>>(_path: P) -> Result<Self> {
        unimplemented!()
    }
}
