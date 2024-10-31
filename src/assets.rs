use std::rc::Rc;

use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;

use crate::modelo::{Modelo720Code, Registro2Modelo720, Shares};

pub struct AssetDifference {
    pub valuation: Decimal,
    pub shares: Shares,
}

pub trait AssetWithValuation {
    fn isin(&self) -> &str;
    fn valuation(&self) -> Decimal;
    fn shares(&self) -> Shares;
    fn country_of_deposit(&self) -> &str;
    fn description(&self) -> &str;
    fn modelo_720_code(&self) -> Modelo720Code;

    fn price_per_share(&self) -> Decimal {
        self.valuation() / self.shares().0
    }

    fn shares_as_cents(&self) -> i64 {
        (self.shares().0 * Decimal::new(100, 0))
            .round_dp_with_strategy(0, rust_decimal::RoundingStrategy::MidpointAwayFromZero)
            .to_i64()
            .unwrap()
    }

    fn valuation_as_cents(&self) -> i64 {
        (self.valuation() * Decimal::new(100, 0))
            .round_dp_with_strategy(0, rust_decimal::RoundingStrategy::MidpointAwayFromZero)
            .to_i64()
            .unwrap()
    }

    fn modelo_720_registro(&self, ejercicio: i16, nif: &str, name: &str) -> Registro2Modelo720 {
        let registro = Registro2Modelo720::new(
            ejercicio,
            nif.to_string(),
            name.to_string(),
            self.country_of_deposit().to_string(),
        );
        let code = self.modelo_720_code();
        Registro2Modelo720 {
            clave_representacion_valores: Some('A'),
            clave_identificacion: Some(1),
            identificacion_valores: Some(self.isin().to_string()),
            clave_tipo_bien: Some(code.code),
            subclave_tipo_bien: Some(code.subcode),
            identificacion_entidad: Some(self.description().to_uppercase()),
            codigo_pais_entidad: Some(self.isin()[..2].to_string()),
            origen_bien_derecho: Some('M'),
            ..registro
        }
    }
}

pub struct Etf {
    pub isin: String,
    pub euro_valuation: Decimal,
    pub shares: Decimal,
    pub deposit_country: String,
    pub description: String,
}

impl AssetWithValuation for Etf {
    fn isin(&self) -> &str {
        &self.isin
    }

    fn valuation(&self) -> Decimal {
        self.euro_valuation
    }

    fn shares(&self) -> Shares {
        Shares(self.shares)
    }

    fn country_of_deposit(&self) -> &str {
        &self.deposit_country
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn modelo_720_code(&self) -> Modelo720Code {
        Modelo720Code {
            code: 'I',
            subcode: 0,
        }
    }
}

pub fn asset_difference(
    left: &dyn AssetWithValuation,
    right: &dyn AssetWithValuation,
) -> AssetDifference {
    AssetDifference {
        valuation: left.valuation() - right.valuation(),
        shares: Shares(left.shares().0 - right.shares().0),
    }
}

pub struct MintosNote {
    pub isin: String,
    pub euro_valuation: Decimal,
    // acquisition_date: NaiveDate,
    pub deposit_country: String,
    pub description: String,
}

impl AssetWithValuation for MintosNote {
    fn isin(&self) -> &str {
        &self.isin
    }

    fn valuation(&self) -> Decimal {
        self.euro_valuation
    }

    fn shares(&self) -> Shares {
        // shares is implicitly the same as the valuation
        Shares(self.valuation())
    }

    fn country_of_deposit(&self) -> &str {
        &self.deposit_country
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn modelo_720_code(&self) -> Modelo720Code {
        Modelo720Code {
            code: 'V',
            subcode: 2,
        }
    }
}

impl AssetWithValuation for &'_ Rc<dyn AssetWithValuation> {
    fn isin(&self) -> &str {
        self.as_ref().isin()
    }

    fn valuation(&self) -> Decimal {
        self.as_ref().valuation()
    }

    fn shares(&self) -> Shares {
        self.as_ref().shares()
    }

    fn country_of_deposit(&self) -> &str {
        self.as_ref().country_of_deposit()
    }

    fn description(&self) -> &str {
        self.as_ref().description()
    }

    fn modelo_720_code(&self) -> Modelo720Code {
        self.as_ref().modelo_720_code()
    }
}

#[derive(Default)]
pub struct Portfolio {
    // TODO: This should definitely be private
    pub assets: Vec<Rc<dyn AssetWithValuation>>,
}

impl Portfolio {
    pub fn from_assets(assets: Vec<Rc<dyn AssetWithValuation>>) -> Portfolio {
        let mut result = Portfolio { assets };
        result
            .assets
            .sort_by(|a, b| a.isin().partial_cmp(b.isin()).unwrap());
        result
    }

    pub fn merge(mut self, other: Portfolio) -> Self {
        self.assets.extend_from_slice(&other.assets);
        self.assets
            .sort_by(|a, b| a.isin().partial_cmp(b.isin()).unwrap());
        // TODO: Add safety check
        self
    }
}
