use std::io::Write;
use std::io::{stdin};
use std::path::{PathBuf};
use std::ffi::OsStr;
use tokio::net::TcpStream;
use tokio::io::{AsyncWriteExt, AsyncReadExt};
use std::fmt::Error;
use tokio_native_tls::TlsConnector;
use native_tls::TlsConnector as NativeTlsConnector;

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
    pub websites: Vec<String>,
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

        print!("Write a path to urls: ");
        std::io::stdout().flush().unwrap();

        
        match stdin().read_line(&mut answer){
            Ok(_) => {},
            Err(_) => return Err(ScrapingError::ErrorReadingUserInput),
        }
        Ok(PathBuf::from(answer.trim()))
    }

    pub fn read_file(&mut self) -> Result<(), ScrapingError> {
        let path = Self::getting_path()?;
        
        let content = match std::fs::read_to_string(&path){
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

pub async fn scrape(website: String) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut chopped = website
        .trim_start_matches("https://")
        .trim_start_matches("http://")
        .splitn(2, '/');

    let (host, path) = (chopped.next().unwrap_or(&website), chopped.next().unwrap_or(""));
    let native_connector = NativeTlsConnector::new()?;
    let connector = TlsConnector::from(native_connector);

    let stream = TcpStream::connect(format!("{}:443", host)).await?;
    let mut tls_stream = connector.connect(host, stream).await?;
    
    let request = format!(
        "GET /{} HTTP/1.1\r\nHost: {}\r\nConnection: close\r\nAccept-Encoding: identity\r\n\r\n",
        path, host
    );

    tls_stream.write_all(request.as_bytes()).await?;
    
    let mut response = String::new();
    tls_stream.read_to_string(&mut response).await?;
    
    let mut file = if let Ok(f) = tokio::fs::File::create(format!("{}.txt", host)).await{
        f
    }else{
        return Err(Box::new(Error));
    };
    file.write_all(response.as_bytes()).await?;
    Ok(())
}
