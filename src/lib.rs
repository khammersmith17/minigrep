use std::env;
use std::error::Error;
use std::fs;

pub struct SearchResult {
    line_text: String,
    line_number: usize,
}

pub struct Config {
    query: String,
    file_path: String,
    ignore_case: bool,
}

impl Config {
    pub fn build(mut args: impl Iterator<Item = String>) -> Result<Config, &'static str> {
        args.next();

        let query = match args.next() {
            Some(arg) => arg,
            None => return Err("Didn't get a query"),
        };

        let file_path = match args.next() {
            Some(arg) => arg,
            None => return Err("Didn't get a file_path"),
        };

        let ignore_case = env::var("IGNORE_CASE").is_ok();

        Ok(Config {
            query,
            file_path,
            ignore_case,
        })
    }
}

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let contents = fs::read_to_string(config.file_path)?;

    let results = if config.ignore_case {
        search_case_insensitive(&config.query, &contents)
    } else {
        search(&config.query, &contents)
    };

    for line in results {
        println!("{0}: {1}", line.line_number, line.line_text);
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
