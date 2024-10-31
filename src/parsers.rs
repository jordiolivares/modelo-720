use std::path::Path;
use std::rc::Rc;

use clap::ValueEnum;
use rust_decimal::Decimal;
use serde::Deserialize;

use crate::assets::{AssetWithValuation, Etf, MintosNote, Portfolio};

#[derive(Debug, Deserialize)]
struct IbkrStatementEntry {
    #[serde(rename = "Description")]
    description: String,
    #[serde(rename = "ISIN")]
    isin: String,
    #[serde(rename = "Quantity")]
    quantity: Decimal,
    #[serde(rename = "PositionValue")]
    position_value: Decimal,
}

pub fn parse_ibkr_statement(path: &Path) -> std::io::Result<Portfolio> {
    let mut reader = csv::Reader::from_path(path)?;
    let mut assets: Vec<Rc<dyn AssetWithValuation>> = Vec::new();
    for row in reader.deserialize() {
        let ibkr_entry: IbkrStatementEntry = row?;
        assets.push(Rc::new(Etf {
            isin: ibkr_entry.isin,
            euro_valuation: ibkr_entry.position_value,
            shares: ibkr_entry.quantity,
            deposit_country: "US".to_string(),
            description: ibkr_entry.description,
        }));
    }
    Ok(Portfolio::from_assets(assets))
}

#[derive(Debug, Deserialize)]
struct MintosStatementEntry {
    #[serde(rename = "ISIN")]
    isin: String,
    #[serde(rename = "Outstanding Principal")]
    pending_principal: Decimal,
    // acquisition_date: NaiveDate,
}

pub fn parse_mintos_statement(path: &Path) -> std::io::Result<Portfolio> {
    let mut reader = csv::Reader::from_path(path)?;
    let mut assets: Vec<Rc<dyn AssetWithValuation>> = Vec::new();
    for row in reader.deserialize() {
        let mintos_entry: MintosStatementEntry = row?;
        assets.push(Rc::new(MintosNote {
            description: format!("MINTOS NOTE {}", mintos_entry.isin),
            // acquisition_date: mintos_entry.acquisition_date,
            isin: mintos_entry.isin,
            euro_valuation: mintos_entry.pending_principal,
            deposit_country: "LV".to_string(),
        }));
    }
    Ok(Portfolio::from_assets(assets))
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum SupportedBrokers {
    InteractiveBrokers,
    Mintos,
}
