use std::{io, net::UdpSocket, time::{Duration, Instant}};

use crossbeam_channel::Receiver;

use crate::{CLIENT_TIME_OUT_MS, PING_MESSAGE, PING_TIMEOUT_SEC, SERVER_HOST, SERVER_UDP_PORT, StockQuote, StockQuoteSet, 
    error::{ERR_PING_READING, QuoutesError}};

pub struct Broadcaster {
    socket: UdpSocket,
}

impl Broadcaster {
    pub fn new() -> Result<Self, QuoutesError> {
        let socket = UdpSocket::bind(format!{"{}:{}", SERVER_HOST, SERVER_UDP_PORT})?;
        socket.set_nonblocking(true)?;
        Ok(Self { socket })
    }

    pub fn send_to(
        &self,
        quotes: &StockQuoteSet,
        target_addr: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let encoded = serde_binary_adv::Serializer::to_bytes(quotes, false)?;
        self.socket.send_to(&encoded, target_addr)?;
        Ok(())
    }

    pub fn start_broadcasting(
        self,
        rx: Receiver<Vec<StockQuote>>,
        tickers_to_send: Vec<String>,
        target_addr: String
    ) -> Result<(), QuoutesError> {

        let timeout = Duration::from_secs(PING_TIMEOUT_SEC);
        let mut last_ping_time = Instant::now();
        let mut buf = [0u8; 64];    

        let mut is_stream_run = false;

        loop {
            if is_stream_run {
                loop {
                    match self.socket.recv_from(&mut buf) {
                        Ok((n, _)) => {
                            let msg = std::str::from_utf8(&buf[..n]).unwrap_or("").trim();
                            if msg.eq_ignore_ascii_case(PING_MESSAGE) {
                                println!("Ping has got");
                                last_ping_time = Instant::now();
                            }
                        }
                        Err(e) if e.kind() == io::ErrorKind::WouldBlock => break,
                        Err(e) => {
                            eprintln!("{}: {}", ERR_PING_READING, e); 
                            break;
                        }
                    }
                }

                if last_ping_time.elapsed() > timeout {
                    println!("Timeout has happend for {}", target_addr); 
                    break;
                }
            }

            match rx.recv_timeout(Duration::from_millis(CLIENT_TIME_OUT_MS)) {
                Ok(quotes) => {
                    let mut quotes_set = StockQuoteSet::new();
                    quotes_set.set_quotes(quotes);
                    let filtered = &quotes_set.filter(tickers_to_send.to_vec());
                    println!("Котировки направлены клиенту: {}", target_addr);
                    self.send_to(filtered, &target_addr)?;
                    if !is_stream_run {is_stream_run = true};
                }
                Err(crossbeam_channel::RecvTimeoutError::Timeout) => {}
                Err(crossbeam_channel::RecvTimeoutError::Disconnected) => break,
            }
        }
        Ok(())
    }
}