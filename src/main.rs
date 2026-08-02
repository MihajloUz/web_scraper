use web_scraper;
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut f = web_scraper::ToScrape::new();
    match f.read_file(){
        Ok(_) => {},
        Err(value) => {
            return Err(format!("{:?}", value).into());
        },

    }
   
    let mut handles = Vec::new();

    for website in f.websites{
       handles.push(tokio::spawn(web_scraper::scrape(website))); 
    } 
    for handle in handles {
        match handle.await {
            Ok(Ok(())) => {}, 
            Ok(Err(e)) => eprintln!("scrape error: {}", e),
            Err(e) => eprintln!("task panicked: {}", e),
        }
    }
    Ok(())
}


