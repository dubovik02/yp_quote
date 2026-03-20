use std::io;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Write;
use std::net::Shutdown;
use std::net::TcpStream;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;

use crate::BASE_COMMAND_PREFIX;
use crate::SUCCESS_SERVER_RESPONSE;
use crate::StockQuote;
use crate::broadcaster::Broadcaster;
use crate::error::ERR_BROADCASTER_CREATE;
use crate::error::ERR_BROADCASTING;
use crate::error::ERR_CONVERTER_PARAMS;
use crate::error::ERR_HOST_PORT_PARAMS;
use crate::error::ERR_STREAMING;
use crate::error::ERR_UNKNOWN_COMMAND;
use crate::error::QuoutesError;

pub fn handle_client(
    stream: &mut TcpStream,  
    clients: Arc<Mutex<Vec<crossbeam_channel::Sender<Vec<StockQuote>>>>>,
) -> Result<(), QuoutesError> {
    
    let stream_c = stream.try_clone()?;
    let mut writer = stream.try_clone()?;
    let mut reader = BufReader::new(stream);

    let greetings = SUCCESS_SERVER_RESPONSE.to_owned() + "\n";
    let _ = writer.write_all(greetings.as_bytes());
    let _ = writer.flush();

    let mut line: String = String::new();
    
    loop {
        line.clear();
        match reader.read_line( &mut line) {
            Ok(0) => {
                // EOF — клиент закрыл соединение
                return Ok(());
            }
            Ok(_) => {
                let input = line.trim();

                if input.is_empty() {
                    let _ = writer.flush();
                    continue;
                }

                let mut parts  = input.split_whitespace();
                let response = match parts.next() {
                    Some(BASE_COMMAND_PREFIX) => {
                        match parts.next() {
                            Some(url) => {
                                match parts.next() {
                                    Some(tickers) => {
                                        let url = url.to_owned();

                                        let tickers = tickers.to_owned();
                                        let tickers_to_send: Vec<_> = 
                                            tickers.trim().split(",").map(|el| el.to_owned()).collect();

                                        if tickers_to_send.is_empty() {return Err(QuoutesError::InvalidTickers)}

                                        let (tx, rx) = crossbeam_channel::unbounded::<Vec<StockQuote>>();
                                        {
                                            let mut guard = clients.lock().unwrap_or_else(|p| p.into_inner());
                                            guard.push(tx);
                                        }

                                        thread::spawn(move || {
                                            match Broadcaster::new() {
                                                Ok(broadcaster) => {
                                                    match broadcaster.start_broadcasting(
                                                        rx,
                                                        tickers_to_send.to_vec(),
                                                        url.to_owned(),
                                                    ) {
                                                        Ok(_) => println!("End to streaming....\n"),
                                                        Err(e) => 
                                                        eprintln!("{}: {}", ERR_BROADCASTING, e), 
                                                    } 
                                                }
                                                Err(e) => 
                                                eprintln!("{}: {}", ERR_BROADCASTER_CREATE, e), 
                                            }
                                        });
                                        "Start to streaming...\n".to_string()
                                    },
                                    None => ERR_HOST_PORT_PARAMS.to_owned() + "\n",
                                }
                            },
                            None => ERR_CONVERTER_PARAMS.to_owned() + "\n",
                        }
                    }

                    Some("EXIT") => {
                        return Ok(());
                    }

                    _ => ERR_UNKNOWN_COMMAND.to_owned() + "\n",
                };

                let _ = writer.write_all(response.as_bytes());
                let _ = writer.flush();
            }
            Err(e) if e.kind() == io::ErrorKind::ConnectionReset => {
                stream_c.shutdown(Shutdown::Both)?;
                return Ok(())
            },
            Err(e) => {
                stream_c.shutdown(Shutdown::Both)?;
                println!("{}: {}", ERR_STREAMING, e);
                return Ok(());
            }
        }
    }
}