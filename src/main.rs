extern crate clap;
extern crate clipboard;
extern crate colored;
extern crate regex;
extern crate reqwest;
extern crate scraper;
extern crate url;

use std::io;
use std::io::{Read, Write};
use clap::{App, Arg};
use clipboard::{ClipboardProvider, ClipboardContext};
use colored::*;
use regex::Regex;
use scraper::{Html, Selector, ElementRef};
use url::percent_encoding::{utf8_percent_encode, DEFAULT_ENCODE_SET};

struct BibTeXReference {
    authors: String,
    title: String,
    original: String
}

impl BibTeXReference {
    fn to_string(&self) -> String {
        format!("{} - {}", self.title, self.authors) 
    }
}

fn main() {
    let matches = App::new("mathbib")
        .arg(Arg::with_name("query")
             .help("An author or paper title to search for")
             .index(1)
             .required(true))
        .get_matches();

    let query = matches.value_of("query").expect("Must provide query.");
    let query_url = url_for_results_page(&query);
    let mut body = String::new();

    reqwest::get(&query_url)
        .expect("Failed to fetch search page for query.")
        .read_to_string(&mut body)
        .expect("Failed to read fetched search page.");

    // TODO: Handle not logged in case.
    let document = Html::parse_document(&body);

    let selector = Selector::parse(".doc pre").unwrap();
    let bib_refs: Vec<BibTeXReference> = document.select(&selector)
        .filter_map(extract_bibtex_reference)
        .collect();

    if bib_refs.len() == 0 {
        println!("Found no articles.");

        return
    }

    println!("Found {} articles: \n", bib_refs.len());

    for (i, bib_ref) in bib_refs.iter().enumerate() {
        println!("{}. {}", i, bib_ref.to_string());
    }

    print!("\nSelect an article: ");
    io::stdout().flush().unwrap();

    let mut selection_input = String::new();

    io::stdin().read_line(&mut selection_input).unwrap();

    let selection: usize = selection_input.trim().parse().unwrap();
    let selected_ref: &BibTeXReference = bib_refs.get(selection)
        .expect("Invalid selection.");

    let mut ctx: ClipboardContext = ClipboardProvider::new().unwrap();

    // TODO: What kind of borrow-fu would avoid cloning?
    if let Ok(_) = ctx.set_contents(selected_ref.original.clone()) {
        println!("{}", "\n✓ BibTeX reference copied to clipboard".green().bold());
    } else {
        panic!("Failed to copy to system clipboard.")
    }
}

fn url_for_results_page(query: &str) -> String {
    let encoded_query = utf8_percent_encode(query, DEFAULT_ENCODE_SET).to_string();

    format!("https://mathscinet-ams-org.proxy.library.reed.edu/mathscinet/search/publications.html?bdlall=&batch_title=Selected+Matches+for%3A+Author%3D%28{e}%29+OR+Title%3D%28{e}%29&pg7=AUCN&yrop=eq&s8=All&pg4=AUCN&co7=AND&co5=AND&s6=&s5={e}&co4=OR&pg5=TI&co6=AND&pg6=PC&s4={e}&arg3=&dr=all&yearRangeFirst=&pg8=ET&s7=&review_format=html&yearRangeSecond=&fmt=bibtex&sort=newest&searchin=", e=encoded_query)
}

fn extract_bibtex_reference(fragment: ElementRef) -> Option<BibTeXReference> {
    let re = Regex::new(r"AUTHOR = \{(?P<author>.+)\}(?s)(.*)(?-s)TITLE = \{(?P<title>.+)\}").unwrap();
    let original = fragment.inner_html();

    if let Some(captures) = re.captures(&original) {
        let authors = String::from(&captures["author"]);
        let title = String::from(&captures["title"]);
        // TODO: Do I really need to compute this again? Can it be borrowed twice?
        let original = String::from(fragment.inner_html().trim());

        Some(BibTeXReference {
            title,
            authors,
            original
        })
    } else {
        None
    }
}
