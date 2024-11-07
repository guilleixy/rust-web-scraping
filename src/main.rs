use reqwest;
use scraper::{Html, Selector};

struct Movie {
    id: i32,
    title: String,
    year: i32,
    image: String,
    pages: u8,
}

struct Rating {
    rating: Option<u8>,
    review: Option<String>,
}

const URL_PREFIX: &str = "https://www.filmaffinity.com/es/reviews/";

fn scrap_review_page(id: i32, page: u8) {
    let url = format!("{}{}/{}.html", URL_PREFIX, page, id);
    let response = reqwest::blocking::get(&url).unwrap();
    if response.status().is_success() {
        let data = response.text().unwrap();
        let mut ratings: Vec<Rating> = Vec::new();
        let html_review_selector = Selector::parse("div.fa-shadow.movie-review-wrapper.rw-item").unwrap();
        let rating_selector = Selector::parse("div.user-reviews-movie-rating").unwrap();
        let review_selector = Selector::parse("div.review-text1").unwrap();
        let document = Html::parse_document(&data);
        let html_reviews = document.select(&html_review_selector);
        for element in html_reviews {
            let rating_text = element.select(&rating_selector).next().map(|e| e.text().collect::<Vec<_>>().join(" ").trim().to_string());
            let rating = rating_text.as_ref().and_then(|text| text.parse::<u8>().ok());
            let review_text = element.select(&review_selector).next().map(|e| e.text().collect::<Vec<_>>().join(" ").trim().to_string());
            let review = review_text;
            ratings.push(Rating { rating, review });
        }

        for rating in ratings {
            println!("Rating: {:?}, Review: {:?}", rating.rating, rating.review);
        }
    } else {
        println!("Failed to fetch the review page: {}", url);
    }
}

fn scrap_film(id: i32, pages: u8) {
    if pages > 0 {
        for i in 0..pages {
            scrap_review_page(id, i + 1);
        }
    } else {
        println!("No pages to scrape for film ID: {}", id);
    }
}

fn get_pages(id: i32) -> u8 {
    let url = format!("{}1/{}.html", URL_PREFIX, id);
    let response = reqwest::blocking::get(&url).unwrap();
    if response.status().is_success() {
        let data = response.text().unwrap();
        let page_selector = Selector::parse("div.pager a").unwrap();
        let document = Html::parse_document(&data);
        if let Some(last_page) = document.select(&page_selector).last() {
            return last_page.text().collect::<Vec<_>>().join(" ").trim().parse::<u8>().unwrap_or(1);
        }
    }
    1
}

fn get_top_films(movies: &mut Vec<Movie>) {
    let response = reqwest::blocking::get("https://www.filmaffinity.com/es/topgen.php").unwrap();
    let data = response.text().unwrap();
    let movie_selector = Selector::parse("div.movie-card.mc-flex.movie-card-0").unwrap();
    let document = Html::parse_document(&data);
    let movies_html = document.select(&movie_selector);

    for element in movies_html {
        let id = element.value().attr("data-movie-id").unwrap().parse::<i32>().unwrap();
        let title = element.select(&Selector::parse("div.mc-title a").unwrap()).next().unwrap().text().collect::<Vec<_>>().join(" ").trim().to_string();
        let year = element.select(&Selector::parse("span.mc-year").unwrap()).next().unwrap().text().collect::<Vec<_>>().join(" ").trim().parse::<i32>().unwrap();
        let image = element.select(&Selector::parse("div.mc-poster img").unwrap()).next().unwrap().value().attr("src").unwrap().to_string();
        let pages = get_pages(id);
        movies.push(Movie { id, title, year, image, pages });
    }

    for movie in movies {
        println!("ID: {}, Title: {}, Year: {}, Image: {}, Pages: {}", movie.id, movie.title, movie.year, movie.image, movie.pages);
    }
}

fn main() {
    let mut movies: Vec<Movie> = Vec::new();
    get_top_films(&mut movies);
    for movie in &movies {
        scrap_film(movie.id, movie.pages);
    }
}