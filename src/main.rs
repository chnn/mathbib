extern crate clap;
extern crate clipboard;
extern crate colored;
#[macro_use] extern crate lazy_static;
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

const MAX_LINE_LENGTH: usize = 99;

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
    let bib_refs = find_bibtex_references(query);

    if bib_refs.len() == 0 {
        println!("Found no articles. Do you have access to MathSciNet?");

        return
    }

    println!("Found {} articles: \n", bib_refs.len());

    print_bibtex_references(&bib_refs);

    print!("\nSelect an article: ");
    io::stdout().flush().unwrap();

    let mut selection_input = String::new();
    io::stdin().read_line(&mut selection_input).unwrap();

    let selection: usize = selection_input.trim().parse().unwrap();
    let selected_ref: &BibTeXReference = bib_refs.get(selection - 1)
        .expect("Invalid selection.");

    copy_article_to_clipboard(selected_ref);

    println!("{}", "\n✓ BibTeX reference copied to clipboard".green().bold());
}

fn find_bibtex_references(query: &str) -> Vec<BibTeXReference> {
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

    bib_refs
}

fn url_for_results_page(query: &str) -> String {
    let encoded_query = utf8_percent_encode(query, DEFAULT_ENCODE_SET).to_string();

    format!("https://mathscinet-ams-org.proxy.library.reed.edu/mathscinet/search/publications.html?bdlall=&batch_title=Selected+Matches+for%3A+Author%3D%28{e}%29+OR+Title%3D%28{e}%29&pg7=AUCN&yrop=eq&s8=All&pg4=AUCN&co7=AND&co5=AND&s6=&s5={e}&co4=OR&pg5=TI&co6=AND&pg6=PC&s4={e}&arg3=&dr=all&yearRangeFirst=&pg8=ET&s7=&review_format=html&yearRangeSecond=&fmt=bibtex&sort=newest&searchin=", e=encoded_query)
}

fn extract_bibtex_reference(fragment: ElementRef) -> Option<BibTeXReference> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"AUTHOR = \{(?P<author>.+)\}(?s)(.*)(?-s)TITLE = \{(?P<title>.+)\}").unwrap();
    }

    let original = fragment.inner_html();

    if let Some(captures) = RE.captures(&original) {
        let authors = String::from(&captures["author"]);
        let title = String::from(&captures["title"]);

        Some(BibTeXReference {
            title,
            authors,
            original: original.trim().to_owned()
        })
    } else {
        None
    }
}

fn print_bibtex_references(bib_refs: &Vec<BibTeXReference>) {
    for (id, bib_ref) in bib_refs.iter().enumerate().map(|(i, b)| (i + 1, b)) {
        let title_author = bib_ref.to_string();

        let id_label = if id < 10 {
            format!(" {}", id).bold()
        } else {
            format!("{}", id).bold()
        };

        if title_author.len() > MAX_LINE_LENGTH - 7 {
            println!("{}. {}...", id_label, &title_author[0..MAX_LINE_LENGTH - 7])
        } else {
            println!("{}. {}", id_label, title_author)
        };
    }
}

fn copy_article_to_clipboard(bib_ref: &BibTeXReference) {
    let mut ctx: ClipboardContext = ClipboardProvider::new().unwrap();

    ctx.set_contents(bib_ref.original.to_owned())
        .expect("Failed to copy reference to clipboard");
}
