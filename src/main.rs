use std::{
    path::{Path, PathBuf},
    rc::Rc,
};

use clap::{command, Parser, Subcommand};
use modelo_720_rust::{
    assets::{asset_difference, AssetWithValuation, Portfolio},
    modelo::{Modelo720, Shares},
    parsers::{parse_ibkr_statement, parse_mintos_statement, SupportedBrokers},
};
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;

struct FullJoinIterator<I: Iterator> {
    is_initialized: bool,
    left: I,
    last_left: Option<I::Item>,
    right: I,
    last_right: Option<I::Item>,
}

impl<T, I: Iterator<Item = T>> FullJoinIterator<I>
where
    T: AssetWithValuation + Clone,
{
    fn new(left: I, right: I) -> Self {
        FullJoinIterator {
            is_initialized: false,
            left,
            last_left: None,
            right,
            last_right: None,
        }
    }
}

enum JoinResult<I: Iterator> {
    OuterLeft(I::Item),
    Inner(I::Item, I::Item),
    OuterRight(I::Item),
}

impl<T, I: Iterator<Item = T>> Iterator for FullJoinIterator<I>
where
    T: AssetWithValuation + Clone,
{
    type Item = JoinResult<I>;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.is_initialized {
            self.last_left = self.left.next();
            self.last_right = self.right.next();
            self.is_initialized = true;
        }
        match (self.last_left.clone(), self.last_right.clone()) {
            (None, None) => None,
            (None, Some(right)) => {
                self.last_right = self.right.next();
                Some(JoinResult::OuterRight(right))
            }
            (Some(left), None) => {
                self.last_left = self.left.next();
                Some(JoinResult::OuterLeft(left))
            }
            (Some(left), Some(right)) => {
                if left.isin() < right.isin() {
                    self.last_left = self.left.next();
                    Some(JoinResult::OuterLeft(left))
                } else if left.isin() == right.isin() {
                    self.last_left = self.left.next();
                    self.last_right = self.right.next();
                    Some(JoinResult::Inner(left, right))
                } else {
                    self.last_right = self.right.next();
                    Some(JoinResult::OuterRight(right))
                }
            }
        }
    }
}

enum PortfolioChange {
    NewAcquisition(Rc<dyn AssetWithValuation>),
    Changed(Rc<dyn AssetWithValuation>, Rc<dyn AssetWithValuation>),
    Sold(Rc<dyn AssetWithValuation>),
}

fn compute_modelo720(
    ejercicio: i16,
    nif: &str,
    name: &str,
    phone: i64,
    current: &Portfolio,
    previous: &Portfolio,
) -> Modelo720 {
    let left = current.assets.iter();
    let right = previous.assets.iter();
    let iterator = FullJoinIterator::new(left, right);
    let entries = iterator
        .map(|result| match result {
            JoinResult::OuterLeft(left) => PortfolioChange::NewAcquisition(left.clone()),
            JoinResult::Inner(left, right) => PortfolioChange::Changed(left.clone(), right.clone()),
            JoinResult::OuterRight(right) => PortfolioChange::Sold(right.clone()),
        })
        .flat_map(|change| match change {
            PortfolioChange::NewAcquisition(acquisition) => {
                let mut registro = acquisition.modelo_720_registro(ejercicio, nif, name);
                registro.origen_bien_derecho = Some('A');
                registro.numero_valores = Some(acquisition.shares());
                registro.valoracion1 = Some(acquisition.valuation_as_cents());
                vec![registro]
            }
            PortfolioChange::Changed(new_value, old_value) => {
                let diff = asset_difference(new_value.as_ref(), old_value.as_ref());

                let current_price_per_share = new_value.price_per_share();
                if diff.shares.0 > Decimal::ZERO {
                    // If we have more shares then we modify the value of what we have and add a new entry for the acquisition.
                    let mut previous_registro = old_value.modelo_720_registro(ejercicio, nif, name);
                    previous_registro.origen_bien_derecho = Some('M');
                    previous_registro.numero_valores = Some(old_value.shares());
                    Some(old_value.shares_as_cents());
                    previous_registro.valoracion1 = (old_value.shares().0
                        * current_price_per_share
                        * Decimal::new(100, 0))
                    .round_dp_with_strategy(0, rust_decimal::RoundingStrategy::MidpointAwayFromZero)
                    .to_i64();

                    let mut new_registro = new_value.modelo_720_registro(ejercicio, nif, name);
                    new_registro.origen_bien_derecho = Some('A');
                    new_registro.numero_valores = Some(diff.shares);
                    new_registro.valoracion1 = (diff.shares.0
                        * current_price_per_share
                        * Decimal::new(100, 0))
                    .round_dp_with_strategy(0, rust_decimal::RoundingStrategy::MidpointAwayFromZero)
                    .to_i64();

                    vec![previous_registro, new_registro]
                } else if diff.shares.0 == Decimal::ZERO {
                    // If instead there are no new shares then we just revalue what we have.
                    let mut current_registro = new_value.modelo_720_registro(ejercicio, nif, name);
                    current_registro.origen_bien_derecho = Some('M');
                    vec![current_registro]
                } else {
                    // If we have less shares then we revalue what remains and then add an entry for the sale. Total sales are already handled in registro2Sold.
                    let mut current_registro = new_value.modelo_720_registro(ejercicio, nif, name);
                    current_registro.origen_bien_derecho = Some('M');

                    let mut sale_registro = current_registro.clone();
                    sale_registro.origen_bien_derecho = Some('C');
                    sale_registro.numero_valores = Some(Shares(diff.shares.0.abs()));
                    sale_registro.valoracion1 = (sale_registro.numero_valores.unwrap().0.abs()
                        * current_price_per_share
                        * Decimal::new(100, 0))
                    .round_dp_with_strategy(0, rust_decimal::RoundingStrategy::MidpointAwayFromZero)
                    .to_i64();
                    vec![current_registro, sale_registro]
                }
            }
            PortfolioChange::Sold(old_value) => {
                let mut registro = old_value.modelo_720_registro(ejercicio, nif, name);
                registro.origen_bien_derecho = Some('C');
                registro.numero_valores = Some(old_value.shares());
                vec![registro]
            }
        })
        .collect();
    Modelo720::new(ejercicio, nif, name, phone, entries)
}

#[derive(Debug, Clone, Subcommand)]
enum Commands {
    Concat {
        #[arg(short, long)]
        left: PathBuf,

        #[arg(short, long)]
        right: PathBuf,

        #[arg(short, long)]
        out: PathBuf,
    },
    Generate {
        #[arg(value_enum)]
        broker: SupportedBrokers,

        #[arg(long)]
        previous_statement: Option<PathBuf>,

        #[arg(long)]
        current_statement: PathBuf,

        #[arg(long)]
        fiscal_year: i16,

        #[arg(long)]
        name: String,

        #[arg(long)]
        nif: String,

        #[arg(long)]
        phone: i64,

        #[arg(short, long)]
        out: PathBuf,
    },
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    subcommand: Commands,
}

fn concat_modelo_720(left: &Path, right: &Path) -> Modelo720 {
    let mut a = Modelo720::from_path(left);
    let b = Modelo720::from_path(right);
    a.concat(b);
    a
}

fn main() {
    let cli = Args::parse();
    let x = cli.subcommand;
    match x {
        Commands::Concat { left, right, out } => {
            let result = concat_modelo_720(&left, &right);
            result.save_to_file(&out);
        }
        Commands::Generate {
            broker,
            previous_statement,
            current_statement,
            fiscal_year,
            name,
            nif,
            phone,
            out,
        } => {
            let (previous_portfolio, current_portfolio) = match broker {
                SupportedBrokers::InteractiveBrokers => {
                    let previous = previous_statement
                        .and_then(|x| parse_ibkr_statement(&x).ok())
                        .unwrap_or(Portfolio::default());
                    let current = parse_ibkr_statement(&current_statement).unwrap();
                    (previous, current)
                }
                SupportedBrokers::Mintos => {
                    let previous = previous_statement
                        .and_then(|x| parse_mintos_statement(&x).ok())
                        .unwrap_or(Portfolio::default());
                    let current = parse_mintos_statement(&current_statement).unwrap();
                    (previous, current)
                }
            };
            let modelo720 = compute_modelo720(
                fiscal_year,
                &nif,
                &name,
                phone,
                &current_portfolio,
                &previous_portfolio,
            );
            modelo720.save_to_file(&out);
        }
    }
}
