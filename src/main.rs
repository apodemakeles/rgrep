use std::{
    io::{BufRead, BufReader, Result},
    path::Path,
};

use clap::Parser;
use colored::*;
use regex::Regex;
use tokio::fs::File;

fn grep<'a, 'b>(r: &'a Regex, input: &'b str) -> Option<Vec<Segment<'b>>> {
    let matches = r.find_iter(input);
    let m: Vec<(usize, usize)> = matches.map(|m| (m.start(), m.end())).collect();
    if m.is_empty() {
        return None;
    }
    let mut v: Vec<Segment<'b>> = Vec::new();
    let mut last: usize = 0;
    let mut char_last = 0;
    for ele in m {
        if ele.0 > last {
            let text = &input[last..ele.0];
            v.push(Segment::Text(text));
            char_last += text.chars().count();
        }
        let keyword = &input[ele.0..ele.1];
        v.push(Segment::Keyword(Keyword {
            text: keyword,
            char_start: char_last + 1,
        }));
        char_last += keyword.chars().count();
        last = ele.1;
    }
    if last < input.len() + 1 {
        v.push(Segment::Text(&input[last..]));
    }

    Some(v)
}

fn print_if_match<'a, 'b>(r: &Regex, input: &str, line: usize) {
    if let Some(segments) = grep(r, input) {
        let char_indies = segments
            .iter()
            .filter(|s| match s {
                Segment::Keyword(_) => true,
                _ => false,
            })
            .map(|s| match s {
                Segment::Keyword(keyword) => keyword.char_start.to_string(),
                _ => panic!("error"),
            })
            .collect::<Vec<_>>()
            .join(",");
        print!("{}-{}: ", line, char_indies);
        for segment in segments {
            match segment {
                Segment::Text(s) => print!("{}", s),
                Segment::Keyword(keyword) => print!("{}", keyword.text.red()),
            }
        }
        print!("{}", "\n");
    }
}

async fn grep_file(r: &Regex, path: impl AsRef<Path>) -> Result<()> {
    println!("{}:", path.as_ref().display());

    let file = File::open(path).await?;
    let file = file.into_std().await;

    let mut num = 0;
    for line_res in BufReader::new(file).lines() {
        num += 1;
        print_if_match(r, &line_res.unwrap(), num);
    }

    Ok(())
}

#[derive(Parser, Debug)]
struct Args {
    pattern: String,
    file: String,
}

#[derive(Debug, PartialEq, Eq)]
enum Segment<'a> {
    Text(&'a str),
    Keyword(Keyword<'a>),
}

#[derive(Debug, PartialEq, Eq)]
struct Keyword<'a> {
    text: &'a str,
    char_start: usize,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let r = Regex::new(&args.pattern).unwrap();
    grep_file(&r, &args.file).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{grep, Keyword, Segment};
    use regex::Regex;

    #[test]
    fn text_match_should_work() {
        let r = Regex::new(r"一只").unwrap();
        let input = "这里有一只鸟,那里有一只鱼。";
        let v = grep(&r, input);
        let segments = v.unwrap();
        assert_eq!(5, segments.len());
        assert_eq!(Segment::Text("这里有"), segments[0]);
        assert_eq!(
            Segment::Keyword(Keyword {
                text: "一只",
                char_start: 4
            }),
            segments[1]
        );
        assert_eq!(Segment::Text("鸟,那里有"), segments[2]);
        assert_eq!(
            Segment::Keyword(Keyword {
                text: "一只",
                char_start: 11
            }),
            segments[3]
        );
        assert_eq!(Segment::Text("鱼。"), segments[4]);
    }

    #[test]
    fn regex_match_should_work() {
        let r = Regex::new(r"一\w{2}").unwrap();
        let input = "这里有一只鸟,那里有一只鱼。";
        let v = grep(&r, input);
        let segments = v.unwrap();
        assert_eq!(5, segments.len());
        assert_eq!(Segment::Text("这里有"), segments[0]);
        assert_eq!(
            Segment::Keyword(Keyword {
                text: "一只鸟",
                char_start: 4
            }),
            segments[1]
        );
        assert_eq!(Segment::Text(",那里有"), segments[2]);
        assert_eq!(
            Segment::Keyword(Keyword {
                text: "一只鱼",
                char_start: 11
            }),
            segments[3]
        );
        assert_eq!(Segment::Text("。"), segments[4]);
    }
}
