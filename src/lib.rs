use std::io::{stdin};
use std::fs;
use std::path::{PathBuf};
use std::ffi::OsStr;
use std::net::TcpStream;
use native_tls::TlsConnector;
use std::io::{Read, Write};


#[derive(Debug)]
pub enum ScrapingError{
    ErrorReadingUserInput,
    ErrorReadingFile,
    ErrorNotAValidExtension,
    ErrorTlsConnector,
    ErrorTcpConnection,
    ErrorStreamConnection,
    ErrorWriteDataToStream,
    ErrorReadingDataFromStream,
    ErrorCreatingAFile,
    ErrorWritingToAFile,
}

enum FileType{
    Unknown,
    CSV, 
    JSON,
}
pub struct ToScrape{
    websites: Vec<String>,
    file_type: FileType,
}
impl ToScrape{
    pub fn new() -> Self{
        Self{
            websites: Vec::new(),
            file_type: FileType::Unknown,
        }
    }

    fn getting_path() -> Result<PathBuf, ScrapingError>{
        let mut answer = String::new();

        println!("Write a path to urls: ");

        match stdin().read_line(&mut answer){
            Ok(_) => {},
            Err(_) => return Err(ScrapingError::ErrorReadingUserInput),
        }
        Ok(PathBuf::from(answer.trim()))
    }

    pub fn read_file(&mut self) -> Result<(), ScrapingError> {
        let path = Self::getting_path()?;
        
        let content = match fs::read_to_string(&path){
            Ok(value) => {value},
            Err(_) => return Err(ScrapingError::ErrorReadingFile),
        }; 
        match path.extension() {
            Some(ext) if ext == OsStr::new("csv") => {
                self.file_type = FileType::CSV;
                self.websites = content
                    .trim()
                    .split('\n')
                    .filter(|s| !s.is_empty())
                    .map(|s| s.replace('"', ""))
                   .collect();
            }
            Some(ext) if ext == OsStr::new("json") => {
                self.file_type = FileType::JSON;
                self.websites = content
                    .trim()
                    .trim_matches(|c| c == '[' || c == ']')
                    .split(',')
                    .map(|s| s.trim().trim_matches(' '))
                    .filter(|s| !s.is_empty())
                    .map(|s| s.replace('"', ""))
                    .map(String::from)
                    .collect();
            }
            _ => return Err(ScrapingError::ErrorNotAValidExtension), 
        }
        Ok(())
    } 
}

pub struct Scraping{
    data: Vec<String>, 
}

impl Scraping{
    pub fn new() -> Self{
        Self{
            data: Vec::new(),
        }
    }
    pub fn scrape(&mut self, info: &ToScrape) -> Vec<(String, ScrapingError)>{
        let mut errors: Vec<(String, ScrapingError)> = Vec::new(); 
        for website in &info.websites{

            let mut chopped = website
                .trim_start_matches("https://")
                .trim_start_matches("http://")
                .splitn(2, '/');

            let (host, path) = (chopped.next().unwrap_or(website), chopped.next().unwrap_or(""));
            
            let connector = match TlsConnector::new() {
                Ok(value) => {value},
                Err(_) => {
                    errors.push((host.to_string(), ScrapingError::ErrorTlsConnector));
                    continue;
                },
            };
            let stream = match TcpStream::connect(format!("{}:443", host.to_string())){
                Ok(value) => {value},
                Err(_) => {
                    errors.push((host.to_string(), ScrapingError::ErrorTcpConnection));
                    continue;
                },
            };  
            let mut stream = match connector.connect(host, stream){
                Ok(value) => {value},
                Err(_) => {
                    errors.push((host.to_string(), ScrapingError::ErrorStreamConnection));
                    continue;
                },
            };

            let request = format!(
                "GET /{} HTTP/1.1\r\nHost: {}\r\nConnection: close\r\nAccept-Encoding: identity\r\n\r\n",
                path, host
            );

            match stream.write_all(request.as_bytes()){
                Ok(_) => {},
                Err(_) => {
                    errors.push((host.to_string(), ScrapingError::ErrorWriteDataToStream));
                    continue;
                },
            }
            
            let mut response = String::new();
            match stream.read_to_string(&mut response){
                Ok(_) => {},
                Err(_) => {
                    errors.push((host.to_string(), ScrapingError::ErrorReadingDataFromStream));
                    continue;
                },
            }
            
            let mut file = if let Ok(f) = std::fs::File::create(format!("{}.txt", host)){
                f
            }else{
                errors.push(("".to_string(), ScrapingError::ErrorCreatingAFile));
                continue;
            };
            match file.write_all(response.as_bytes()){
                Ok(_) => {},
                Err(_) => {
                    errors.push(("".to_string(), ScrapingError::ErrorWritingToAFile));
                    continue;
                },
            };
            self.data.push(response.clone());
        }
        errors
    }
}
