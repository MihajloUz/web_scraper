use web_scraper;
//use tokio;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut f = web_scraper::ToScrape::new();
    match f.read_file(){
        Ok(_) => {},
        Err(value) => {
            return Err(format!("{:?}", value).into());
        },

    }
    let mut d = web_scraper::Scraping::new();
    
    let scrape_output = d.scrape(&f);
    
    if !scrape_output.is_empty(){
        for e in scrape_output{
            println!("{:?}", e);
        }
    }

    Ok(())
}


