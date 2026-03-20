use std::{net::TcpListener, sync::{Arc, Mutex}, thread, time::Duration};
use yp_quote::{GENERATE_QUOTES_PERIOD, SERVER_HOST, SERVER_PORT, StockQuote, 
    error::{ERR_CONNECTION_FAILED, ERR_SERVICE_CONNECTION, ERR_START_STOCK_GENERATOR, QuoutesError}, 
    quotes_generator::generate_quotes, server::handle_client};

fn main() -> Result<(), QuoutesError> {
    let listener = TcpListener::bind(format!("{}:{}", SERVER_HOST, SERVER_PORT))?;
    println!("Server has started on port {}", SERVER_PORT);

    let (tx, rx) = crossbeam_channel::unbounded::<Vec<StockQuote>>();

    let clients: Arc<Mutex<Vec<crossbeam_channel::Sender<Vec<StockQuote>>>>> = Arc::new(Mutex::new(Vec::new()));

    thread::spawn(move || {
        let mut counter = 0;
        loop {
            match generate_quotes(vec!["*"]) {
                Ok(val) => {
                    let val_c = val.to_vec();
                    match tx.send(val) {
                        Ok(_) => {
                            counter += 1;
                            println!("Котировки сгенерированы и переданы в канал {} раз", counter);
                            thread::sleep(Duration::from_secs(GENERATE_QUOTES_PERIOD));
                        },
                        Err(_) => {
                            QuoutesError::SenderError(val_c.iter().map(|item| item.to_string() + "\n").collect());
                        }
                    }; 
                }
                Err(e) => eprintln!("{}: {}", ERR_START_STOCK_GENERATOR, e),
            }
        }
    });

    let rx_clone = rx.clone();
    let clients_clone = Arc::clone(&clients); 
    thread::spawn(move || {
        while let Ok(quotes) = rx_clone.recv() {
            let mut guard = clients_clone.lock().unwrap_or_else(|p| p.into_inner());
            guard.retain(|tx| tx.send(quotes.clone()).is_ok());
        }
    });

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {  
                let clients_clone = Arc::clone(&clients); 
                thread::spawn(move || {
                    match handle_client(&mut stream, clients_clone) {
                        Ok(_) => (),
                        Err(e) => eprintln!("{}: {}", ERR_SERVICE_CONNECTION, e),
                    };
                });
            }
            Err(e) => eprintln!("{}: {}", ERR_CONNECTION_FAILED, e),
        }
    }

    Ok(())
}
 