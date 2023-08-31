use std::path::PathBuf;
use std::{path::Path, fs::DirEntry};
use std::fs;
use crate::parsing::{FileParsingError, read_file};
use crate::lexer::Lexer;
use std::collections::HashMap;
use serde_derive::{Serialize, Deserialize};

struct Stats {
    total_docs_number: u32,
    skipped_docs: u32,
}

impl Stats {
    fn new() -> Self {
        Self {
            total_docs_number: 0,
            skipped_docs: 0,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Default)]
struct Document {
    tfs: HashMap<String, f32>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Indexer {
    documents: HashMap<PathBuf, Document>,
    doc_freq: HashMap<String, f32>,
}

fn compute_cos_sim(a: &Vec<f32>, b: &Vec<f32>) -> f32 {
    let norm_a_sq = compute_dot_product(a, a).sqrt();
    let norm_b_sq = compute_dot_product(b, b).sqrt();

    let dot_a_b = compute_dot_product(a, b);


    dot_a_b / (norm_a_sq * norm_b_sq)
}

fn compute_dot_product(a: &Vec<f32>, b: &Vec<f32>) -> f32 {
    a.iter().zip(b.iter()).map(|(a, b)| a * b).sum()
}

impl Indexer {
    pub fn get_docs_for_query(&self, query: &str) -> Vec<(PathBuf, f32)> {

        let idfs = self.compute_idfs_query(query);
        let tf_idf_query = self.compute_tf_idf_query(query, &idfs);

        let mut docs = self.documents.keys()
            .map(|doc| {
                let tf_idf = self.compute_tf_idf_query_doc(doc, query, &idfs);
                (doc.to_owned(), compute_cos_sim(&tf_idf, &tf_idf_query))
            })
            .filter(|(_, v)| !v.is_nan())
            .collect::<Vec<_>>();
        

        docs.sort_by(|(doc_a, a), (doc_b, b)|{
            let ordering = b.partial_cmp(a).unwrap();
            if ordering.is_eq() {
                doc_a.to_str().unwrap().cmp(doc_b.to_str().unwrap())
            }
            else {
                ordering
            }
        });

        docs

    }

    fn compute_tf_idf_term(&self, doc: &PathBuf, term: &String, idf: f32) -> f32 {
        let tf = self.documents
                        .get(doc)
                        .unwrap()
                        .tfs
                        .get(term)
                        .and_then(|x| Some(*x))
                        .unwrap_or(0.0);

        tf * idf

    }

    fn compute_tf_idf_query_doc(&self, doc: &PathBuf, query: &str, idfs: &HashMap<String, f32>) -> Vec<f32> {
        Lexer::new(query)
            .map(|term| self.compute_tf_idf_term(
                doc, &term, 
                idfs.get(&term).and_then(|x| Some(*x)).unwrap()))
            .collect()
    }

    fn compute_idf_term(&self, term: &str) -> f32 {
        let docs_number = self.documents.len() as f32 + 1.0;

        let docs_number_for_term = 
                self.doc_freq.get(term)
                    .and_then(|x| Some(*x))
                    .unwrap_or(0.0) + 1.0;

        (docs_number / docs_number_for_term).log10()
    }

    fn compute_idfs_query(&self, query: &str) -> HashMap<String, f32> {
        Lexer::new(query)
            .map(|term| {
                let idf_term = self.compute_idf_term(&term);
                (term, idf_term)
            })
            .collect()
    }

    fn compute_tf_idf_query(&self, query: &str, idfs: &HashMap<String, f32>) -> Vec<f32> {

        let tfs = compute_tfs(query);

        Lexer::new(query)
        .map(|term| {

            let tf = tfs.get(&term).unwrap();
            let idf = idfs.get(&term).unwrap();
            
            tf * idf
        })
        .collect()
    }
}


fn compute_tfs(content: &str) -> HashMap<String, f32> {
    let mut terms_freqs = HashMap::new();
    for term in Lexer::new(&content) {
        if let Some(freq) = terms_freqs.get_mut(&term) {
            *freq += 1.0;
        }
        else {
            terms_freqs.insert(term, 1.0);
        }
    }

    let sum = terms_freqs.values().sum::<f32>();
    terms_freqs.values_mut().for_each(|v| *v /= sum);

    terms_freqs
}


fn index_file<P>(file_path: &P, indexer: &mut Indexer) -> Result<(), FileParsingError>
where
    P: AsRef<Path>
{
    // take file content
    let file_content = read_file(file_path)?;
    // tokenize the file into terms
    // for each term compute terms frequency
    let terms_freqs = compute_tfs(&file_content);

    // compute update document requency for each term

    for term in terms_freqs.keys() {

        if let Some(f) = indexer.doc_freq.get_mut(term) {
            *f += 1.0;
        }
        else {
            indexer.doc_freq.insert(term.to_string(), 1.0);
        }
    }

    let document = Document {
        tfs: terms_freqs,
    };

    indexer.documents.insert(file_path.as_ref().to_path_buf(), document);

    Ok(())
}

pub fn index_dir<P>(dir_path: &P, indexer: &mut Indexer) -> Result<(), ()>
where
    P: AsRef<Path>
{
    let mut dir_entries = fs::read_dir(dir_path).map_err(|e| {
        eprintln!("ERROR: {e}");
    })?;

    let mut stats = Stats::new();

    let mut sub_dirs = Vec::new();

    loop {
        for entry in dir_entries.by_ref() {
            match entry {
                Ok(entry) => {
                    handle_entry(entry, &mut sub_dirs, indexer, &mut stats);
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

    println!("Indexing stats:");
    println!("Dir total documents: {}", stats.total_docs_number);
    println!("Indexded documents {}", indexer.documents.len());
    println!("Skipped documents: {}", stats.skipped_docs);


    Ok(())
}

fn handle_entry(entry: DirEntry, sub_dirs: &mut Vec<PathBuf>, indexer: &mut Indexer, stats: &mut Stats) {
    if let Ok(file_type) = entry.file_type() {
        if file_type.is_dir() {
            sub_dirs.push(entry.path());
        }

        else {
            let path = entry.path();
            println!("Indexing {}", path.display());
            stats.total_docs_number += 1;

            if indexer.documents.contains_key(&path) {
                stats.skipped_docs += 1;
                return;
            }

            if let Err(e) = index_file(&path, indexer) {
                stats.skipped_docs += 1;
                eprintln!("{e}");
            }
        }
    }
}