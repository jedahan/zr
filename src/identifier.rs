use core::str::Split;
use std::fmt;
use std::iter::Take;
use std::path::PathBuf;
use url::{ParseError, Url};

#[derive(Debug, PartialEq)]
pub struct Identifier(url::Url);

impl fmt::Display for Identifier {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "{}", self.0)?;
        Ok(())
    }
}

impl From<String> for Identifier {
    fn from(string: String) -> Identifier {
        let has_base = Url::parse(&string) != Err(ParseError::RelativeUrlWithoutBase);

        // author/name[/path/to/file.zsh] -> https://github.com/author/name[.git/path/]
        if !has_base {
            if !string.contains('/') {
                panic!("'{}' is not a valid identifier", string)
            }
            let string = format!("https://github.com/{}", string);
            return Identifier(Url::parse(&string).unwrap());
        }

        Identifier(Url::parse(&string).unwrap())
    }
}

/// An Identifier is just a url::Url with some nice helper methods
impl Identifier {
    /// Where should the files be stored?
    pub fn filepath(&self) -> Result<PathBuf, String> {
        let url = &self.0;
        let segments: Split<char> = url.path_segments().ok_or_else(|| "no path")?;

        if Url::host_str(&url) == Some("github.com") {
            let strip_author_and_name = segments.skip(2).collect::<Vec<_>>();
            return Ok(PathBuf::from(strip_author_and_name.join("/")));
        }

        Ok(PathBuf::new())
    }

    /// Get the original Identifier string
    pub fn source(&self) -> String {
        self.0.to_string()
    }

    fn segments(&self) -> Take<Split<char>> {
        let url = &self.0;
        let mut segments: Split<char> = url.path_segments().unwrap();

        let path_index = match Url::host_str(&url) {
            Some("github.com") => 2,
            _ => segments
                .position(|segment| segment.ends_with(".git"))
                .unwrap(),
        };

        segments.take(path_index)
    }

    /// Get the full repository path from an Identifier
    pub fn repository(&self) -> String {
        let segments = self.segments();
        let repository_path: Vec<String> = segments.map(String::from).collect();

        let mut url = self.0.clone();
        url.set_path(&repository_path.join("/"));
        url.to_string()
    }

    /// What is the name of the plugin
    pub fn name(&self) -> String {
        self.segments()
            .last()
            .unwrap()
            .trim_end_matches(".git")
            .to_string()
    }
}
