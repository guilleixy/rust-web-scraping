use reqwest;
use scraper::{Html, Selector};

struct Film {
    title: Option<String>,
    description: Option<String>,
    url: Option<String>,
    image: Option<String>,
    ratings: Option<Vec<Rating>>,
}

struct Rating {
    rating: Option<f32>,
    review: Option<String>,
}

fn main() {
    let response = reqwest::blocking::get("https://www.filmaffinity.com/en/film257164.html");
    let data = response.unwrap().text().unwrap();
    println!("{data}");
    let mut films: Vec<Film> = Vec::new();
    let html_film_selector = scraper::Selector::parse("div").unwrap();
    let document = Html::parse_document(&data);
    let html_films = document.select(&html_film_selector);

}
