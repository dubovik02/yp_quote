
use crate::{StockQuote, error::QuoutesError};

pub fn generate_quotes(tickers: Vec<&str>) -> Result<Vec<StockQuote>, QuoutesError> {

    let mut result: Vec<StockQuote> = Vec::new();

    let tickers_vec: Vec<&str> = vec![
        "AAPL","MSFT","GOOGL","AMZN","NVDA","META","TSLA","JPM","JNJ","V","PG","UNH","HD","DIS","PYPL",
        "NFLX","ADBE","CRM","INTC","CSCO","PFE","ABT","TMO","ABBV","LLY","PEP","COST","TXN","AVGO","ACN",
        "QCOM","DHR","MDT","NKE","UPS","RTX","HON","ORCL","LIN","AMGN","LOW","SBUX","SPGI","INTU","ISRG",
        "T","BMY","DE","PLD","CI","CAT","GS","UNP","AMT","AXP","MS","BLK","GE","SYK","GILD","MMM","MO","LMT",
        "FISV","ADI","BKNG","C","SO","NEE","ZTS","TGT","DUK","ICE","BDX","PNC","CMCSA","SCHW","MDLZ","TJX",
        "USB","CL","EMR","APD","COF","FDX","AON","WM","ECL","ITW","VRTX","D","NSC","PGR","ETN","FIS","PSA",
        "KLAC","MCD","ADP","APTV","AEP","MCO","SHW","DD","ROP","SLB","HUM","BSX","NOC","EW"
    ];
    
    for item in tickers_vec {
        let ticker = item.to_string();
        if tickers.contains(&ticker.as_str()) || tickers.contains(&"*") {
            let mut quote = StockQuote::new();
            quote.generate_indicators(&ticker)?;
            result.push(quote);
        }
    }

    Ok(result)
}