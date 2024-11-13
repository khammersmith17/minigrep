use fancy::colorize;
use std::env::{current_dir, var};
use std::error::Error;
use std::fs::read_to_string;
use std::io::{stdout, Write};
use std::sync::mpsc::channel;
use std::thread::spawn;
use walkdir::{DirEntry, WalkDir};

#[derive(Default, Debug)]
pub struct SearchResult {
    line_text: String,
    line_number: usize,
}

#[derive(Default, Debug)]
pub struct Config {
    query: String,
    file_paths: Vec<String>,
    ignore_case: bool,
}

pub struct FileMeta {
    file_name: String,
    file_contents: String,
}

pub struct FileResults {
    file_name: String,
    search_result: Vec<SearchResult>,
}

impl Config {
    pub fn build(mut args: impl Iterator<Item = String>) -> Result<Config, &'static str> {
        args.next();

        let query = match args.next() {
            Some(arg) => arg,
            None => return Err("Didn't get a query"),
        };

        let file_paths = match args.next() {
            Some(arg) => vec![arg],
            None => {
                let current_directory = match current_dir() {
                    Ok(dir) => dir,
                    Err(_) => return Err("Error getting the current directory"),
                };

                let current_dir_str = match current_directory.to_str() {
                    Some(path) => path.to_string(),
                    None => return Err("Could not convert Path"),
                };
                let children: Vec<DirEntry> = WalkDir::new(&current_dir_str)
                    .into_iter()
                    .filter_map(|entry| entry.ok())
                    .collect();

                let mut file_paths: Vec<String> = Vec::new();
                /*
                let children = match read_dir(current_dir_str) {
                    Ok(paths) => paths,
                    Err(_) => return Err("Could not get the child paths"),
                };*/

                for file in children.into_iter() {
                    let path = file.path();
                    if !path.is_file() {
                        continue;
                    }
                    let path_str = match path.to_str() {
                        Some(path) => path,
                        None => continue,
                    };
                    file_paths.push(path_str.to_string())
                }
                file_paths
            }
        };

        let ignore_case = var("IGNORE_CASE").is_ok();

        Ok(Config {
            query,
            file_paths,
            ignore_case,
        })
    }
}

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let mut all_file_contents: Vec<FileMeta> = Vec::new();
    for file in config.file_paths {
        // let contents = read_to_string(&file)?;
        if let Ok(contents) = read_to_string(&file) {
            let file_meta = FileMeta {
                file_name: file,
                file_contents: contents,
            };

            all_file_contents.push(file_meta);
        }
    }

    //let mut results: Vec<FileResults> = Vec::new();
    let (sender, results) = channel();
    for file in all_file_contents {
        let local_send = sender.clone();
        let query = config.query.clone();

        spawn(move || {
            let current_result = if config.ignore_case {
                search_case_insensitive(&query, &file.file_contents)
            } else {
                search(&query, &file.file_contents)
            };

            if current_result.len() > 1 {
                local_send
                    .send(FileResults {
                        file_name: file.file_name,
                        search_result: current_result,
                    })
                    .unwrap();
            }
        });
    }

    drop(sender);

    let stdout = stdout();
    let mut handle = stdout.lock();
    for file_hit in results {
        let file = colorize!("[bold|blue]File: [underline|blue]{}", file_hit.file_name);
        write!(handle, "{}\n", file)?;
        for query_hit in file_hit.search_result {
            let result = colorize!(
                "[bold|red]{0}: {1}\n",
                query_hit.line_number,
                query_hit.line_text,
            );
            write!(handle, "{}", result)?;
        }
        write!(handle, "\n")?;
    }

    Ok(())
}

pub fn search<'a>(query: &str, contents: &'a str) -> Vec<SearchResult> {
    let mut results = Vec::new();

    for (i, line) in contents.lines().enumerate() {
        if line.contains(query) {
            results.push(SearchResult {
                line_text: line.to_string(),
                line_number: i,
            });
        }
    }
    results

    //if we were still returning a vec of strings
    //contents
    //  .lines()
    //  .filter(|line| line.contains(query))
    //  .collect()
}

pub fn search_case_insensitive<'a>(query: &str, contents: &'a str) -> Vec<SearchResult> {
    let query = query.to_lowercase();
    let mut results = Vec::new();

    for (i, line) in contents.lines().enumerate() {
        if line.to_lowercase().contains(&query) {
            results.push(SearchResult {
                line_text: line.to_string(),
                line_number: i,
            });
        }
    }

    results
}

//tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_result() {
        let query = "duct";
        let contains = "\
Rust:
safe, fast, productive.
Pick three.";

        assert_eq!(vec!["safe, fast, productive."], search(query, contains));
    }

    #[test]
    fn case_insensitive() {
        let query = "rUsT";
        let contents = "\
Rust:
safe, fast, productive,
Pick three.
Trust me.";

        assert_eq!(
            vec!["Rust:", "Trust me."],
            search_case_insensitive(query, contents)
        );
    }
}
