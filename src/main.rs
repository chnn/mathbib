// TODO: Handle unauthenticated case.

extern crate clap;
extern crate regex;
extern crate reqwest;
extern crate scraper;
extern crate url;

use std::io;
use std::io::Read;
use clap::{App, Arg};
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

    // let query = matches.value_of("query").expect("Must provide query.");
    // let query_url = url_for_search_page(&query);
    // let mut body = String::new();

    // reqwest::get(&query_url)
    //     .expect("Failed to fetch search page for query.")
    //     .read_to_string(&mut body)
    //     .expect("Failed to read fetched search page.");

    // // TODO: Handle not logged in case.
    // let document = Html::parse_document(&body);
    let document = Html::parse_document(&get_html_fixture());

    let selector = Selector::parse(".doc pre").unwrap();
    let bib_refs: Vec<BibTeXReference> = document.select(&selector).map(extract_bibtex_reference).collect();

    if bib_refs.len() == 0 {
        println!("Found no articles.");

        return
    }

    println!("Found {} articles: \n", bib_refs.len());

    for (i, bib_ref) in bib_refs.iter().enumerate() {
        println!("{}. {}", i, bib_ref.to_string());
    }

    println!("\nSelect an article: ");

    let mut selection_input = String::new();

    io::stdin().read_line(&mut selection_input).unwrap();

    let selection: u32 = selection_input.trim().parse().unwrap();
}

fn url_for_results_page(query: &str) -> String {
    let encoded_query = utf8_percent_encode(query, DEFAULT_ENCODE_SET).to_string();

    format!("https://mathscinet-ams-org.proxy.library.reed.edu/mathscinet/search/publications.html?bdlall=&batch_title=Selected+Matches+for%3A+Author%3D%28{e}%29+OR+Title%3D%28{e}%29&pg7=AUCN&yrop=eq&s8=All&pg4=AUCN&co7=AND&co5=AND&s6=&s5={e}&co4=OR&pg5=TI&co6=AND&pg6=PC&s4={e}&arg3=&dr=all&yearRangeFirst=&pg8=ET&s7=&review_format=html&yearRangeSecond=&fmt=bibtex&sort=newest&searchin=", e=encoded_query)
}


fn extract_bibtex_reference(fragment: ElementRef) -> BibTeXReference {
    let re = Regex::new(r"AUTHOR = \{(?P<author>.+)\}(?s)(.*)(?-s)TITLE = \{(?P<title>.+)\}").unwrap();

    let original = fragment.inner_html();
    let captures = re.captures(&original).expect("Failed to parse BibTeX entry.");
    let authors = String::from(&captures["author"]);
    let title = String::from(&captures["title"]);

    BibTeXReference {
	title,
	authors,
	original: fragment.inner_html()
    }
}

use std::fs::File;
use std::path::Path;

fn get_html_fixture() -> String {
    let path = Path::new("/Users/chris/Downloads/test.html");
    let mut file = File::open(&path).unwrap();
    let mut file_contents = String::new();

    file.read_to_string(&mut file_contents).unwrap();

    file_contents
}

