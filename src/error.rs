use thiserror::Error;
use std::{io, time::SystemTimeError};

/// Сообщение об ошибке чтения
pub const ERR_READ_MSG: &str = "I\\O error while reading from data source";
/// Сообщение об ошибке подключения к сервису
pub const ERR_SERVICE_CONNECTION: &str = "Error while creating stream service";
/// Сообщение об ошибке подключения
pub const ERR_CONNECTION_FAILED: &str = "Connection failed";
/// Сообщение об ошибке указания параметров при запуске стриминга
pub const ERR_CONVERTER_PARAMS: &str = "Usage STREAM <host:port> <Ticker,Ticker, ... | * >";
/// Сообщение об ошибке указания хоста и порта
pub const ERR_HOST_PORT_PARAMS: &str = "You have to set <host:port>";
/// Сообщение о неизвестной команлде
pub const ERR_UNKNOWN_COMMAND: &str = "Unknown command";
/// Сообщение об ошибке стриминга
pub const ERR_STREAMING: &str = "Error while streaming";
/// Сообщение об ошибке запуска сервиса генерации котировок
pub const ERR_START_STOCK_GENERATOR: &str = "Quotes generator has not started";
/// Сообщение об ошибке пинга
pub const ERR_PING_READING: &str = "Error while reading a ping";
/// Сообщение об ошибке пинга
pub const ERR_PING_SENDING: &str = "Error while sending a ping";
/// Сообщение об ошибке ввода
pub const ERR_INVALID_UDP_INPUT: &str = "Invalid UDP adress";
/// Сообщение об ошибке сервера
pub const ERR_SERVER_RESPONSE: &str = "Server could'nt set connection";
/// Сообщение об ошибке чтения UDP
pub const ERR_UDP_RECIVE: &str = "Server could'nt set connection";
/// Сообщение об ошибке десириализации
pub const ERR_DESERIALIZE: &str = "Invalid json data of quotes";
/// Сообщение об ошибке броадкастинга
pub const ERR_BROADCASTING: &str = "Error while start broadcasting";
/// Сообщение об ошибке броадкастинга
pub const ERR_BROADCASTER_CREATE: &str = "Error while create broadcaster";

/// Ошибки
#[derive(Error, Debug)]
pub enum QuoutesError {
    /// Ошибки ввода-вывода
    #[error("Input-output error: {0}")]
    Io(#[from] io::Error),
    /// Ошибки парсинга
    #[error("Sereliazation or desereliazation error: {0}")]
    Binary(#[from] serde_binary_adv::BinaryError),
    /// Ошибка отправки по каналу
    #[error("Sender error. Message has not sent: {0}")]
    SenderError(String),
    /// Ошибки парсинга даты
    #[error("Parsing date/time error: {0}")]
    DateParse(#[from] SystemTimeError),
    /// Ошибка уквзвния тикера для стриминга
    #[error("You have to set tickers for streaming or \"*\" for streaming all tickers.")]
    InvalidTickers,
    /// Ошибки ввода
    #[error("Input data error. Data {0} is incorrect")]
    InvalidInput(String),
    /// Иные ошибки
    #[error("Unknown error {0}")]
    Unknown(#[from] Box<dyn std::error::Error>)
}