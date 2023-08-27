use std::{env, process::ExitCode, io::{Write, BufWriter, BufReader}, env::Args, path::Path, fs::File};

mod indexer;
mod parsing;
mod lexer;

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
        let f = File::open(&cmd_args.index_file_name).unwrap();

        let reader = BufReader::new(f);

        serde_json::from_reader(reader).unwrap()
    }
    else {
        let mut indexer = Default::default();

        if let Err(_) = indexer::index_dir(&cmd_args.docs_folder, &mut indexer) {
            return ExitCode::FAILURE;
        }

        let f = File::create(&cmd_args.index_file_name).unwrap();        
        let writer = BufWriter::new(f);

        serde_json::to_writer(writer, &indexer).unwrap();

        indexer
    };

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

    ExitCode::SUCCESS
}
