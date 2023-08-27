use std::path::PathBuf;
use std::{path::Path, fs::DirEntry};
use std::fs;
use crate::parsing::{FileParsingError, read_file};
use crate::lexer::Lexer;
use std::collections::HashMap;
use serde_derive::{Serialize, Deserialize};

pub type Document = PathBuf;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Indexer {
    tf_doc: HashMap<Document, HashMap<String, f32>>,
    doc_freq: HashMap<String, f32>,
}

impl Indexer {
    pub fn get_docs_for_query(&self, query: &str) -> Vec<(Document, f32)> {

        let mut docs_score = self.tf_doc.keys()
            .map(|doc| (doc.to_owned(), self.compute_tf_idf_query(doc, query)))
            .collect::<Vec<_>>();
        
        docs_score.sort_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap());
        docs_score.reverse();
        docs_score
    }

    fn compute_tf_idf_term(&self, doc: &Document, term: &String) -> f32 {
        let tf = self.tf_doc.get(doc)
                                            .unwrap()
                                            .get(term)
                                            .and_then(|x| Some(*x))
                                            .unwrap_or(0.0);

        let docs_number = self.tf_doc.len() as f32;

        let docs_number_for_term = 
                self.doc_freq.get(term)
                    .and_then(|x| Some(*x))
                    .unwrap_or(1.0);

        let idf = (docs_number / docs_number_for_term).log10();

        tf * idf

    }

    fn compute_tf_idf_query(&self, doc: &Document, query: &str) -> f32 {
        let lexer = Lexer::new(query);

        lexer.map(|term| self.compute_tf_idf_term(doc, &term)).sum()
    }
}


fn index_file<P>(file_path: &P, indexer: &mut Indexer) -> Result<(), FileParsingError>
where
    P: AsRef<Path>
{
    // take file content
    let file_content = read_file(file_path)?;
    // tokenize the file into terms
    // for each term compute terms frequency
    let mut terms_freqs = HashMap::new();
    for term in Lexer::new(&file_content) {
        if let Some(freq) = terms_freqs.get_mut(&term) {
            *freq += 1.0;
        }
        else {
            terms_freqs.insert(term, 1.0);
        }
    }

    let sum = terms_freqs.values().sum::<f32>();
    terms_freqs.values_mut().for_each(|v| *v /= sum);

    // compute update document requency for each term

    for term in terms_freqs.keys() {

        if let Some(f) = indexer.doc_freq.get_mut(term) {
            *f += 1.0;
        }
        else {
            indexer.doc_freq.insert(term.to_string(), 1.0);
        }
    }

    indexer.tf_doc.insert(file_path.as_ref().to_path_buf(), terms_freqs);

    Ok(())
}

pub fn index_dir<P>(dir_path: &P, indexer: &mut Indexer) -> Result<(), ()>
where
    P: AsRef<Path>
{
    let mut dir_entries = fs::read_dir(dir_path).map_err(|e| {
        eprintln!("ERROR: {e}");
    })?;

    let mut sub_dirs = Vec::new();

    loop {
        for entry in dir_entries.by_ref() {
            match entry {
                Ok(entry) => {
                    handle_entry(entry, &mut sub_dirs, indexer);
                }
                Err(e) => {
                    eprintln!("ERROR: {e}")
                }
            }
        }

        if let Some(dir_path) = sub_dirs.pop() {
            dir_entries = match fs::read_dir(dir_path) {
                Ok(entries) => entries,
                Err(_) => continue
            }
        }
        else {
            break;
        }

    }


    Ok(())
}

fn handle_entry(entry: DirEntry, sub_dirs: &mut Vec<PathBuf>, indexer: &mut Indexer) {
    if let Ok(file_type) = entry.file_type() {
        if file_type.is_dir() {
            sub_dirs.push(entry.path());
        }

        else {

            let path = entry.path();
            println!("Indexing {}", path.display());
            if let Err(e) = index_file(&path, indexer) {
                eprintln!("{e}");
            }
        }
    }
}