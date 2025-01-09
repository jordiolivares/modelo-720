use std::collections::HashMap;
use std::path::Path;
use std::rc::Rc;

use clap::ValueEnum;
use once_cell::sync::Lazy;
use regex::Regex;
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
    #[serde(alias = "Principal pendiente")]
    pending_principal: Decimal,
}

#[derive(Debug, Deserialize, PartialEq)]
enum PaymentType {
    #[serde(rename = "Principal received")]
    #[serde(alias = "Capital recibido")]
    #[serde(alias = "Ingresos del principal recibidos por la recompra del préstamo")]
    #[serde(alias = "Principal recibido por la recompra de partes pequeñas de préstamos")]
    #[serde(alias = "Principal received from loan repurchase")]
    #[serde(alias = "Principal received from repurchase of small loan parts")]
    PrincipalReceived,
    #[serde(rename = "Investment")]
    Investment,
    #[serde(rename = "Secondary market transaction")]
    #[serde(alias = "Operación del Mercado Secundario")]
    SecondaryMarketTransaction,
    #[serde(untagged)]
    Unknown(String),
}

#[derive(Debug, Deserialize)]
struct MintosActivityStatementEntry {
    #[serde(rename = "Details")]
    #[serde(alias = "Detalles")]
    details: String,
    #[serde(rename = "Turnover")]
    #[serde(alias = "Volumen de negocios")]
    turnover: Decimal,
    #[serde(rename = "Payment Type")]
    #[serde(alias = "Tipo de pago")]
    payment_type: PaymentType,
}

impl MintosActivityStatementEntry {
    fn isin(&self) -> Option<&str> {
        static ISIN_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new("LV\\w{9}\\d").unwrap());
        ISIN_REGEX.find(&self.details).map(|x| x.as_str())
    }
}

// Mintos current portfolio statement may not reflect the state from a previous point in time.
// This method reverses operation from the activity statement in order to reconstruct the previous state by applying the inverse operation.
pub fn parse_mintos_statement_with_reverted_changes(
    statement_path: &Path,
    activity_statement_path: &Path,
) -> std::io::Result<Portfolio> {
    let current_portfolio = parse_mintos_statement_as_is(statement_path)?;
    let mut isin_notes = HashMap::new();
    for note in current_portfolio.assets {
        isin_notes.insert(note.isin().to_string(), note);
    }
    let mut reader = csv::Reader::from_path(activity_statement_path)?;
    for row in reader.deserialize() {
        let parsed: MintosActivityStatementEntry = row?;
        match parsed.payment_type {
            PaymentType::Unknown(_) => continue, // We ignore activity that doesn't affect the principal.
            _ => {}
        };
        let isin = match parsed.isin() {
            Some(x) => x,
            None => continue, // This is a legacy loan without ISIN, as such it can be ignored.
        };
        if !isin_notes.contains_key(isin) {
            isin_notes.insert(
                isin.to_string(),
                Rc::new(MintosNote::new(isin.to_string(), Decimal::from(0))),
            );
        }
        let old_value = isin_notes[isin].clone();
        isin_notes.insert(
            isin.to_string(),
            // turnover is positive when we've received capital and negative when making an investment, these are the signs we want for reversing the operations.
            Rc::new(MintosNote::new(
                isin.to_string(),
                old_value.valuation() + parsed.turnover,
            )),
        );
    }
    let fixed_portfolio: Vec<Rc<dyn AssetWithValuation>> = isin_notes.values().cloned().collect();
    Ok(Portfolio::from_assets(fixed_portfolio))
}

pub fn parse_mintos_statement(path: &Path) -> std::io::Result<Portfolio> {
    if path.is_file() {
        parse_mintos_statement_as_is(path)
    } else {
        parse_mintos_statement_with_reverted_changes(
            &path.join("statement.csv"),
            &path.join("activity.csv"),
        )
    }
}

pub fn parse_mintos_statement_as_is(path: &Path) -> std::io::Result<Portfolio> {
    let mut reader = csv::Reader::from_path(path)?;
    let mut assets: Vec<Rc<dyn AssetWithValuation>> = Vec::new();
    for row in reader.deserialize() {
        let mintos_entry: MintosStatementEntry = row?;
        assets.push(Rc::new(MintosNote::new(
            mintos_entry.isin,
            mintos_entry.pending_principal,
        )));
    }
    Ok(Portfolio::from_assets(assets))
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum SupportedBrokers {
    InteractiveBrokers,
    Mintos,
}
