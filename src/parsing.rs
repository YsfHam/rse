use std::{io, path::Path, fs, fmt::Display};

use html_parser::{Dom, Node};

#[derive(Debug)]
pub enum FileParsingError {
    ReadFileError(io::Error),
    ParseHtmlError(html_parser::Error),

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

    let html_dom = Dom::parse(&html_content)
                        .map_err(|err| FileParsingError::ParseHtmlError(err))?;

    let mut html_text = String::new();

    let mut dom_elements = vec![html_dom.children];

    while let Some(children) = dom_elements.pop() {
        for node in children {
            match node {
                Node::Text(text) | Node::Comment(text)
                 => {
                     html_text.push_str(&text);
                     html_text.push(' ');
                 } 
                
                Node::Element(element) => dom_elements.push(element.children)
            }
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
            "txt" => return read_text_file(file_path),
            "html" => return read_html_file(file_path),
            ext => return Err(FileParsingError::UnknownExtensionError(ext.to_string()))
        }
    }

    Err(FileParsingError::NoExtensionError)
}
