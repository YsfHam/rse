use std::{env, process::ExitCode, io::{Write, BufWriter, BufReader}, env::Args, path::Path, fs::File};

use tiny_http::{Server, Response};

mod indexer;
mod parsing;
mod lexer;
mod server;

#[derive(Debug)]
struct CmdArgs {
    docs_folder: String,
    index_file_name: String
}

impl CmdArgs {
    fn from_args(mut cmd_args: Args) -> Result<Self, ()> {
        let docs_folder = match cmd_args.next() {
            Some(n) => n,
            None => return Err(())
        };

        let index_file_name = match cmd_args.next() {
            None => {
                let path = Path::new(&docs_folder);
                let mut index_file_name = match path.file_name() {
                    Some(file_name) => file_name.to_string_lossy().to_string(),
                    None => return Err(())
                };
                index_file_name.push_str(".index.json");

                let mut ancestor_folder = path.ancestors().skip(1).next().unwrap().to_string_lossy().to_string();
                ancestor_folder.push('/');

                ancestor_folder.push_str(&index_file_name);
                ancestor_folder
            }
            Some(file_name) => file_name
        };
        
        Ok(
            Self {
                docs_folder,
                index_file_name
            }
        )
    }
}

fn load_indexer_data<P>(path: P) -> indexer::Indexer 
where
    P: AsRef<Path>
{
    let f = File::open(path).unwrap();

    let reader = BufReader::new(f);

    serde_json::from_reader(reader).unwrap()
}

fn index_dir_and_save<P>(dir_path: P, save_file_path: P) -> Result<indexer::Indexer, ()> 
where
    P: AsRef<Path>
{
    let mut indexer = Default::default();

    indexer::index_dir(&dir_path, &mut indexer)?;

    let f = File::create(&save_file_path).unwrap();        
    let writer = BufWriter::new(f);

    serde_json::to_writer(writer, &indexer).unwrap();

    Ok(indexer)
}

fn main() -> ExitCode {

    let mut args = env::args();
    let exec_name = args.next().unwrap();

    let cmd_args = match CmdArgs::from_args(args) {
        Ok(args) => args,
        Err(_) => {
            eprintln!("Usage {} dir_name", exec_name);
            return ExitCode::FAILURE;
        }
    };

    let index_file_path = Path::new(&cmd_args.index_file_name);

    let exists = match index_file_path.try_exists() {
        Ok(exists) => exists,
        Err(_) => return ExitCode::FAILURE
    };

    let indexer = if exists {
        load_indexer_data(&cmd_args.index_file_name)
    }
    else if let Ok(indexer) = index_dir_and_save(&cmd_args.docs_folder, &cmd_args.index_file_name) {
        indexer
    }
    else {
        return ExitCode::FAILURE;
    };

    server::start_server("127.0.0.1:8080", &indexer).unwrap();

    

    ExitCode::SUCCESS
}


fn run_search(indexer: &indexer::Indexer) {
    loop {
        let mut buffer = String::new();

        print!("> ");
        std::io::stdout().flush().unwrap();

        if let Err(_) = std::io::stdin().read_line(&mut buffer) {
            break;
        }

        buffer = buffer.trim_end().to_string();

        if buffer == "!q" {
            break;
        }

        println!("Results for query: {buffer}");

        let docs_list = indexer.get_docs_for_query(&buffer);

        for (doc, score) in docs_list.iter().take(10) {
            println!("{} ===> {}", doc.display(), score);
        }
    }
}
