use std::{io, path::Path, fs, fmt::Display};

use tl;

#[derive(Debug)]
pub enum FileParsingError {
    ReadFileError(io::Error),
    ParseHtmlError(tl::ParseError),

    UnknownExtensionError(String),
    NoExtensionError,
}

impl Display for FileParsingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ERROR: ")?;
        match self {
            FileParsingError::ReadFileError(io_error) => {
                write!(f, "{}", io_error)
            }
            FileParsingError::ParseHtmlError(e) => {
                write!(f, "{}", e)
            }

            FileParsingError::UnknownExtensionError(ext) => {
                write!(f, "Unkown extention {ext}")
            }
            FileParsingError::NoExtensionError => {
                write!(f, "Cannot read file without extension")
            }
        }
    }
}

fn read_html_file<P>(file_path: P) -> Result<String, FileParsingError>
where
    P : AsRef<Path>
{
    let html_content = fs::read_to_string(file_path).map_err(|err| FileParsingError::ReadFileError(err))?;
    let mut html_text = String::new();

    let html_dom = tl::parse(&html_content, Default::default()).map_err(|e| FileParsingError::ParseHtmlError(e))?;

    for node in html_dom.nodes() {
        match node {
            tl::Node::Raw(text)
            => {
                html_text.push_str(&text.as_utf8_str());
                html_text.push(' ');
            },
            _ => {}
        }
    }

    Ok(html_text)

}

fn read_text_file<P>(file_path: P) -> Result<String, FileParsingError>
where
    P : AsRef<Path>
{
    fs::read_to_string(file_path).map_err(|err| FileParsingError::ReadFileError(err))
}

pub fn read_file<P>(file_path: P) -> Result<String, FileParsingError>
where
    P : AsRef<Path>
{
    if let Some(ext) = file_path.as_ref().extension() {
        match ext.to_string_lossy().as_ref() {
            "txt" | "md" => return read_text_file(file_path),
            "html" | "xhtml" => return read_html_file(file_path),
            ext => return Err(FileParsingError::UnknownExtensionError(ext.to_string()))
        }
    }

    Err(FileParsingError::NoExtensionError)
}
