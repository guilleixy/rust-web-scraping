use reqwest;
use scraper::{Html, Selector};
use std::fs::{self, File, OpenOptions};
use std::io::{self, BufRead};
use std::path::Path;

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

// Scrapes a review page for a given movie ID and page number, and writes the reviews to a CSV file.
fn scrap_review_page(id: i32, page: u8, csv_writer: &mut csv::Writer<File>) {
    let url = format!("{}{}/{}.html", URL_PREFIX, page, id);
    let response = reqwest::blocking::get(&url).unwrap();
    if response.status().is_success() {
        let data = response.text().unwrap();
        let mut ratings: Vec<Rating> = Vec::new();
        let html_review_selector = Selector::parse("div.fa-shadow.movie-review-wrapper.rw-item")
        .unwrap();
        let rating_selector = Selector::parse("div.user-reviews-movie-rating").unwrap();
        let review_selector = Selector::parse("div.review-text1").unwrap();
        let document = Html::parse_document(&data);
        let html_reviews = document.select(&html_review_selector);
        for element in html_reviews {
            let rating_text = element.select(&rating_selector).next().map(|e| e.text()
            .collect::<Vec<_>>().join(" ").trim().to_string());
            let rating = rating_text.as_ref().and_then(|text| text.parse::<u8>().ok());
            let review_text = element.select(&review_selector).next().map(|e| e.text()
            .collect::<Vec<_>>().join(" ").trim().to_string());
            let review = review_text;
            ratings.push(Rating { rating, review });
        }

        for rating in ratings {
            println!("Rating: {:?}, Review: {:?}", rating.rating, rating.review);
            csv_writer.write_record(&[
                rating.review.clone().unwrap_or_default(),
                rating.rating.map(|r| r.to_string()).unwrap_or_default(),
            ]).unwrap();
        }
    } else {
        println!("Failed to fetch the review page: {}", url);
    }
}

// Scrapes all review pages for a given movie ID and writes the reviews to a CSV file.
fn scrap_film(id: i32, pages: u8, csv_writer: &mut csv::Writer<File>) {
    if pages > 0 {
        for i in 0..pages {
            scrap_review_page(id, i + 1, csv_writer);
        }
    } else {
        println!("No pages to scrape for film ID: {}", id);
    }
}

// Retrieves the number of pages of reviews for a given movie ID.
fn get_pages(id: i32) -> u8 {
    let url = format!("{}1/{}.html", URL_PREFIX, id);
    let response = reqwest::blocking::get(&url).unwrap();
    if response.status().is_success() {
        let data = response.text().unwrap();
        let page_selector = Selector::parse("div.pager a").unwrap();
        let document = Html::parse_document(&data);
        let mut pages = document.select(&page_selector).collect::<Vec<_>>();
        if pages.len() > 1 {
            if let Some(second_last_page) = pages.iter().nth_back(1) {
                let second_last_page_text = second_last_page.text().collect::<Vec<_>>().join(" ")
                .trim().to_string();
                println!("Second to last page: {}", second_last_page_text);
                return second_last_page_text.parse::<u8>().unwrap_or(1);
            }
        }
    }
    1
}

// Scrapes the top films from the FilmAffinity website and populates the provided vector with movie data.
fn get_top_films(movies: &mut Vec<Movie>) {
    let response = reqwest::blocking::get("https://www.filmaffinity.com/es/topgen.php").unwrap();
    let data = response.text().unwrap();
    let movie_selector = Selector::parse("div.movie-card.mc-flex.movie-card-0").unwrap();
    let document = Html::parse_document(&data);
    let movies_html = document.select(&movie_selector);

    for element in movies_html {
        let id = element.value().attr("data-movie-id").unwrap().parse::<i32>().unwrap();
        let title = element.select(&Selector::parse("div.mc-title a").unwrap()).next().unwrap()
        .text().collect::<Vec<_>>().join(" ").trim().to_string();
        let year = element.select(&Selector::parse("span.mc-year").unwrap()).next().unwrap()
        .text().collect::<Vec<_>>().join(" ").trim().parse::<i32>().unwrap();
        let image = element.select(&Selector::parse("div.mc-poster img").unwrap()).next().unwrap()
        .value().attr("src").unwrap().to_string();
        let pages = get_pages(id);
        movies.push(Movie { id, title, year, image, pages });
    }

    for movie in movies {
        println!("ID: {}, Title: {}, Year: {}, Image: {}, Pages: {}", 
        movie.id, movie.title, movie.year, movie.image, movie.pages);
    }
}

// Scrapes the top films from a specific year from the FilmAffinity website and prints the movie data.
fn get_top_films_from_year() {
    let mut movies: Vec<Movie> = Vec::new();
    let year = 2021;
    let url = format!("https://www.filmaffinity.com/es/topgen.php?genres=&chv=0&orderby=rc&movietype=full%7C&country=&fromyear={}&toyear={}&ratingcount=2&runtimemin=0&runtimemax=7", year, year);
    let response = reqwest::blocking::get(url).unwrap();
    let data = response.text().unwrap();
    let movie_selector = Selector::parse("div.movie-card.mc-flex.movie-card-0").unwrap();
    let document = Html::parse_document(&data);
    let movies_html = document.select(&movie_selector);

    for element in movies_html {
        let id = element.value().attr("data-movie-id").unwrap().parse::<i32>().unwrap();
        let title = element.select(&Selector::parse("div.mc-title a").unwrap()).next().unwrap()
        .text().collect::<Vec<_>>().join(" ").trim().to_string();
        let year = element.select(&Selector::parse("span.mc-year").unwrap()).next().unwrap()
        .text().collect::<Vec<_>>().join(" ").trim().parse::<i32>().unwrap();
        let image = element.select(&Selector::parse("div.mc-poster img").unwrap()).next()
        .unwrap().value().attr("src").unwrap().to_string();
        let pages = get_pages(id);
        movies.push(Movie { id, title, year, image, pages });
    }

    for movie in movies {
        println!("{},{},{},{},{}", movie.id, movie.title, movie.year, movie.image, movie.pages);
    }
}

fn main() {

    //get_top_films_from_year();

    let mut movies: Vec<Movie> = Vec::new();

    if !Path::new("movies.csv").exists() {
        get_top_films(&mut movies);
        let mut csv_writer = csv::Writer::from_path("movies.csv").unwrap();
        csv_writer.write_record(&["ID", "Title", "Year", "Image", "Pages"]).unwrap();
        for movie in &movies {
            csv_writer.write_record(&[
                movie.id.to_string(),
                movie.title.clone(),
                movie.year.to_string(),
                movie.image.clone(),
                movie.pages.to_string(),
            ]).unwrap();
        }
    } else {
        let file = File::open("movies.csv").unwrap();
        let mut rdr = csv::Reader::from_reader(file);
        for result in rdr.records() {
            let record = result.unwrap();
            movies.push(Movie {
                id: record[0].parse().unwrap(),
                title: record[1].to_string(),
                year: record[2].parse().unwrap(),
                image: record[3].to_string(),
                pages: record[4].parse().unwrap(),
            });
        }
    }

    if Path::new("last_review.txt").exists() {
        let last_reviewed_id = fs::read_to_string("last_review.txt").unwrap().trim().parse::<i32>()
        .unwrap();
        if let Some(pos) = movies.iter().position(|movie| movie.id == last_reviewed_id) {
            movies.drain(..=pos);
        }
    }

    let file = OpenOptions::new()
        .write(true)
        .append(true)
        .open("reviews.csv")
        .unwrap();
    let mut csv_writer = csv::Writer::from_writer(file);

    for movie in &movies {
        scrap_film(movie.id, movie.pages, &mut csv_writer);
    }
}