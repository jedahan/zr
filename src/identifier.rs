use std::path::PathBuf;
use url::{ParseError, Url};

#[derive(Clone, Debug, PartialEq)]
pub struct Identifier {
    pub name: String,
    pub url: Url,
    pub dir: PathBuf,
    pub file: Option<String>,
}

impl Identifier {
    pub fn new(name: String) -> Self {
        if !name.contains('/') {
            panic!("'{}' is not a valid identifier", &name)
        }

        let uri = if name.contains(".git") {
            format!("{}.git", name.split(".git").next().unwrap())
        } else {
            name.clone()
        };

        let url = match Url::parse(&uri) {
            Ok(url) => url,
            Err(ParseError::RelativeUrlWithoutBase) => {
                Url::parse(&format!("https://github.com/{}", &uri)).unwrap()
            }
            Err(e) => panic!("{}", e),
        };

        let file = name.split(".git/").nth(1).map(String::from);
        let dir = PathBuf::from(url.path_segments().unwrap().collect::<Vec<_>>().join("/"));
        Identifier {
            name,
            url,
            dir,
            file,
        }
    }
}
