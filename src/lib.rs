use fancy::printcoln;
use std::env::{current_dir, var};
use std::error::Error;
use std::fs::{read_dir, read_to_string};

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

                let mut file_paths: Vec<String> = Vec::new();
                let children = match read_dir(current_dir_str) {
                    Ok(paths) => paths,
                    Err(_) => return Err("Could not get the child paths"),
                };
                for file in children {
                    let file = file.unwrap();
                    let path = file.path();
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
        let contents = read_to_string(&file)?;
        let file_meta = FileMeta {
            file_name: file,
            file_contents: contents,
        };

        all_file_contents.push(file_meta);
    }

    let mut results: Vec<FileResults> = Vec::new();
    for file in all_file_contents {
        let current_result = if config.ignore_case {
            search_case_insensitive(&config.query, &file.file_contents)
        } else {
            search(&config.query, &file.file_contents)
        };

        results.push(FileResults {
            file_name: file.file_name,
            search_result: current_result,
        });
    }

    for file_hit in results {
        printcoln!("[bold|blue]File: [underline|blue]{}", file_hit.file_name);
        for query_hit in file_hit.search_result {
            printcoln!(
                "[bold|red]{0}: {1}",
                query_hit.line_number,
                query_hit.line_text
            );
        }
        println!("\n");
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
