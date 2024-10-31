use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::{fs::File, io::Write};

use chrono::NaiveDate;
use clap::{arg, Parser, Subcommand, ValueEnum};
use fixed_width::Reader;
use fixed_width_derive::FixedWidth;
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;
use serde::de::Visitor;
use serde::{de, Deserialize, Serialize};

#[derive(Clone, Copy, Debug)]
struct Shares(Decimal);

impl Serialize for Shares {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let rounded_to_cents = self
            .0
            .round_dp_with_strategy(2, rust_decimal::RoundingStrategy::MidpointAwayFromZero);
        serializer.serialize_i64((rounded_to_cents * Decimal::new(100, 0)).to_i64().unwrap())
    }
}

struct SharesVisitor;

impl<'de> Visitor<'de> for SharesVisitor {
    type Value = Decimal;

    fn visit_i8<E>(self, value: i8) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Decimal::from(value))
    }

    fn visit_i32<E>(self, value: i32) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Decimal::from(value))
    }

    fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Decimal::from(value))
    }

    fn visit_i16<E>(self, v: i16) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Decimal::from(v))
    }

    fn visit_u8<E>(self, v: u8) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Decimal::from(v))
    }

    fn visit_u16<E>(self, v: u16) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Decimal::from(v))
    }

    fn visit_u32<E>(self, v: u32) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Decimal::from(v))
    }

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Decimal::from(v))
    }

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("Expected a valid decimal number")
    }

    // Similar for other methods:
    //   - visit_i16
    //   - visit_u8
    //   - visit_u16
    //   - visit_u32
    //   - visit_u64
}

impl<'de> Deserialize<'de> for Shares {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer
            .deserialize_i64(SharesVisitor)
            .map(|cents| Shares(cents / Decimal::from(100)))
    }
}

#[derive(Clone, Copy, Debug)]
struct Modelo720Date(Option<NaiveDate>);

impl Serialize for Modelo720Date {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match &self.0 {
            Some(v) => serializer.serialize_str(v.format("%Y%m%d").to_string().as_str()),
            None => serializer.serialize_bytes(&[]),
        }
    }
}

impl<'de> Deserialize<'de> for Modelo720Date {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer
            .deserialize_str(Modelo720DateVisitor)
            .map(|date| Modelo720Date(date))
    }
}

struct Modelo720DateVisitor;

impl<'de> Visitor<'de> for Modelo720DateVisitor {
    type Value = Option<NaiveDate>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("Expected a valid date")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        if v == "00000000" {
            Ok(None)
        } else {
            NaiveDate::parse_from_str(v, "%Y%m%d")
                .map_err(|e| E::custom(e.to_string()))
                .map(Some)
        }
    }
}

#[derive(Clone, Deserialize, Serialize, Debug, FixedWidth)]
struct Registro1Modelo720 {
    #[fixed_width(range = "0..1")]
    tipo: i8,

    #[fixed_width(range = "1..4")]
    modelo_declaracion: i16,

    #[fixed_width(range = "4..8")]
    ejercicio: i16,

    #[fixed_width(range = "8..17")]
    nif_declarante: String,

    #[fixed_width(
        name = "APELLIDOS Y NOMBRE, RAZÓN SOCIAL O DENOMINACIÓN DEL DECLARADO",
        range = "17..57"
    )]
    nombre: String,

    #[fixed_width(name = "TIPO DE SOPORTE", range = "57..58")]
    tipo_soporte: char,

    #[fixed_width(name = "TELEFONO PERSONA CONTACTO", range = "58..67")]
    telefono: i64,

    #[fixed_width(name = "APELLIDOS Y NOMBRE PERSONA CONTACTO", range = "67..107")]
    nombre_persona_contacto: String,

    #[fixed_width(
        name = "NÚMERO IDENTIFICATIVO DE LA DECLARACIÓN",
        range = "107..120",
        justify = "right",
        pad_with = "0"
    )]
    id_declaracion: i64,

    #[fixed_width(name = "DECLARACIÓN COMPLEMENTARIA", range = "120..121")]
    declaracion_complementaria: Option<char>,

    #[fixed_width(name = "DECLARACIÓN SUSTITUTIVA", range = "121..122")]
    declaracion_sustitutiva: Option<char>,

    #[fixed_width(
        name = "NÚMERO IDENTIFICATIVO DE LA DECLARACIÓN ANTERIOR",
        range = "122..135",
        justify = "right",
        pad_with = "0"
    )]
    id_declaracion_anterior: Option<i64>,

    #[fixed_width(
        name = "NÚMERO TOTAL DE REGISTROS DECLARADOS",
        range = "135..144",
        justify = "right",
        pad_with = "0"
    )]
    numero_registros_tipo2: usize,

    #[fixed_width(name = "SUMA TOTAL DE VALORACIÓN 1 (SIGNO)", range = "144..145")]
    valoracion_1_negativa: char,
    #[fixed_width(
        name = "SUMA TOTAL DE VALORACIÓN 1",
        range = "145..162",
        justify = "right",
        pad_with = "0"
    )]
    suma_valoracion1: i64,

    #[fixed_width(name = "SUMA TOTAL DE VALORACIÓN 2 (SIGNO)", range = "162..163")]
    valoracion_2_negativa: char,
    #[fixed_width(
        name = "SUMA TOTAL DE VALORACIÓN 2",
        range = "163..180",
        justify = "right",
        pad_with = "0"
    )]
    suma_valoracion2: i64,

    #[fixed_width(name = "BLANCOS", range = "180..500")]
    blancos: String,
}

impl Registro1Modelo720 {
    fn new(ejercicio: i16, nif: String, nombre: String, telefono: i64) -> Self {
        Registro1Modelo720 {
            tipo: 1,
            modelo_declaracion: 720,
            ejercicio: ejercicio,
            nif_declarante: nif.clone(),
            nombre: nombre.clone(),
            tipo_soporte: 'T',
            telefono,
            nombre_persona_contacto: nombre,
            id_declaracion: 720_000_000_000_0,
            declaracion_complementaria: None,
            declaracion_sustitutiva: None,
            id_declaracion_anterior: None,
            numero_registros_tipo2: 0,
            valoracion_1_negativa: ' ',
            suma_valoracion1: 0,
            valoracion_2_negativa: ' ',
            suma_valoracion2: 0,
            blancos: String::default(),
        }
    }
}

#[derive(Clone, Deserialize, Serialize, Debug, FixedWidth)]
struct Registro2Modelo720 {
    #[fixed_width(range = "0..1")]
    tipo: i8,

    #[fixed_width(range = "1..4")]
    modelo_declaracion: i16,

    #[fixed_width(range = "4..8")]
    ejercicio: i16,

    #[fixed_width(range = "8..17")]
    nif_declarante: String,

    #[fixed_width(range = "17..26")]
    nif_declarado: String,

    #[fixed_width(name = "N.I.F. DEL REPRESENTANTE LEGAL", range = "26..35")]
    nif_representante_legal: Option<String>,

    #[fixed_width(
        name = "APELLIDOS Y NOMBRE, RAZÓN SOCIAL O DENOMINACIÓN DEL DECLARADO",
        range = "35..75"
    )]
    nombre: String,

    #[fixed_width(name = "CLAVE DE CONDICIÓN DEL DECLARANTE", range = "75..76")]
    clave_condicion_declarante: i8,

    #[fixed_width(
        name = "TIPO DE TITULARIDAD SOBRE EL BIEN O DERECHO",
        range = "76..101"
    )]
    tipo_titularidad: Option<String>,

    #[fixed_width(name = "CLAVE TIPO DE BIEN O DERECHO", range = "101..102")]
    clave_tipo_bien: Option<char>,

    #[fixed_width(
        name = "SUBCLAVE DE BIEN O DERECHO",
        range = "102..103",
        justify = "right",
        pad_with = "0"
    )]
    subclave_tipo_bien: Option<i8>,

    #[fixed_width(name = "TIPO DE DERECHO REAL SOBRE INMUEBLE", range = "103..128")]
    tipo_derecho_real_sobre_inmueble: Option<String>,

    #[fixed_width(name = "CÓDIGO DE PAÍS", range = "128..130")]
    codigo_pais: String,

    #[fixed_width(
        name = "CLAVE DE IDENTIFICACIÓN",
        range = "130..131",
        justify = "right",
        pad_with = "0"
    )]
    clave_identificacion: Option<i8>,

    #[fixed_width(name = "IDENTIFICACIÓN DE VALORES", range = "131..143")]
    identificacion_valores: Option<String>,

    #[fixed_width(name = "CLAVE IDENTIFICACIÓN DE CUENTA", range = "143..144")]
    clave_identificacion_cuenta: Option<char>,

    #[fixed_width(name = "CÓDIGO BIC", range = "144..155")]
    codigo_bic: Option<String>,

    #[fixed_width(name = "CÓDIGO DE CUENTA", range = "155..189")]
    codigo_cuenta: Option<String>,

    #[fixed_width(name = "IDENTIFICACIÓN DE LA ENTIDAD", range = "189..230")]
    identificacion_entidad: Option<String>,

    #[fixed_width(
        name = "NÚMERO DE IDENTIFICACIÓN FISCAL EN EL PAÍS DE RESIDENCIA FISCAl",
        range = "230..250"
    )]
    nif_pais_residencia_fiscal: Option<String>,

    #[fixed_width(name = "NOMBRE VÍA PUBLICA Y NÚMERO DE CASA", range = "250..302")]
    nombre_via_publica_entidad: Option<String>,

    #[fixed_width(name = "COMPLEMENTO", range = "302..342")]
    complemento_entidad: Option<String>,

    #[fixed_width(name = "POBLACIÓN/CIUDAD", range = "342..372")]
    poblacion_entidad: Option<String>,

    #[fixed_width(name = "PROVINCIA/REGIÓN/ESTADO", range = "372..402")]
    provincia_entidad: Option<String>,

    #[fixed_width(name = "CÓDIGO POSTAL (ZIP CODE)", range = "402..412")]
    codigo_postal_entidad: Option<String>,

    #[fixed_width(name = "CÓDIGO PAÍS", range = "412..414")]
    codigo_pais_entidad: Option<String>,

    // @FixedFormat(format = "yyyyMMdd")
    #[fixed_width(
        name = "FECHA DE INCORPORACIÓN",
        range = "414..422",
        justify = "right",
        pad_with = "0"
    )]
    fecha_incorporacion: Modelo720Date,

    #[fixed_width(name = "ORIGEN DEL BIEN O DERECHO", range = "422..423")]
    origen_bien_derecho: Option<char>,

    // @FixedFormat(format = "yyyyMMdd")
    #[fixed_width(
        name = "FECHA DE EXTINCIÓN",
        range = "423..431",
        justify = "right",
        pad_with = "0"
    )]
    fecha_extincion: Modelo720Date,

    #[fixed_width(name = "SUMA TOTAL DE VALORACIÓN 1 (SIGNO)", range = "431..432")]
    valoracion_1_negativa: char,
    #[fixed_width(
        name = "Valoracion 1",
        range = "432..446",
        justify = "right",
        pad_with = "0"
    )]
    valoracion1: Option<i64>,

    #[fixed_width(name = "SUMA TOTAL DE VALORACIÓN 1 (SIGNO)", range = "446..447")]
    valoracion_2_negativa: char,
    #[fixed_width(
        name = "Valoracion 2",
        range = "447..461",
        justify = "right",
        pad_with = "0"
    )]
    valoracion2: Option<i64>,

    #[fixed_width(name = "CLAVE DE REPRESENTACIÓN DE VALORES", range = "461..462")]
    clave_representacion_valores: Option<char>,

    #[fixed_width(
        name = "NÚMERO DE VALORES",
        range = "462..474",
        justify = "right",
        pad_with = "0"
    )]
    numero_valores: Option<Shares>,

    #[fixed_width(name = "CLAVE TIPO DE BIEN INMUEBLE", range = "474..475")]
    clave_tipo_bien_inmueble: Option<char>,

    #[fixed_width(
        name = "PORCENTAJE DE PARTICIPACIÓN",
        range = "475..480",
        justify = "right",
        pad_with = "0"
    )]
    porcentaje: i64,

    #[fixed_width(name = "BLANCOS", range = "480..500")]
    blancos: String,
}

impl Registro2Modelo720 {
    fn new(ejercicio: i16, nif: String, nombre: String, codigo_pais: String) -> Self {
        Registro2Modelo720 {
            tipo: 2,
            modelo_declaracion: 720,
            ejercicio: ejercicio,
            nif_declarante: nif.clone(),
            nif_declarado: nif.clone(),
            nif_representante_legal: None,
            nombre: nombre.clone(),
            clave_condicion_declarante: 1,
            tipo_titularidad: None,
            clave_tipo_bien: None,
            subclave_tipo_bien: None,
            tipo_derecho_real_sobre_inmueble: None,
            codigo_pais,
            clave_identificacion: None,
            identificacion_valores: None,
            clave_identificacion_cuenta: None,
            codigo_bic: None,
            codigo_cuenta: None,
            identificacion_entidad: None,
            nif_pais_residencia_fiscal: None,
            nombre_via_publica_entidad: None,
            complemento_entidad: None,
            poblacion_entidad: None,
            provincia_entidad: None,
            codigo_postal_entidad: None,
            codigo_pais_entidad: None,
            fecha_incorporacion: Modelo720Date(None),
            origen_bien_derecho: None,
            fecha_extincion: Modelo720Date(None),
            valoracion_1_negativa: ' ',
            valoracion1: None,
            valoracion_2_negativa: ' ',
            valoracion2: None,
            clave_representacion_valores: None,
            numero_valores: None,
            clave_tipo_bien_inmueble: None,
            porcentaje: 10000,
            blancos: String::default(),
        }
    }
}

struct Modelo720 {
    header: Registro1Modelo720,
    entries: Vec<Registro2Modelo720>,
}

impl Modelo720 {
    fn new(
        ejercicio: i16,
        nif: &str,
        nombre: &str,
        telefono: i64,
        entries: Vec<Registro2Modelo720>,
    ) -> Modelo720 {
        let mut result = Modelo720 {
            header: Registro1Modelo720::new(
                ejercicio,
                nif.to_string(),
                nombre.to_string(),
                telefono,
            ),
            entries,
        };
        result.header.numero_registros_tipo2 = result.entries.len();
        result.header.suma_valoracion1 = result
            .entries
            .iter()
            .map(|x| x.valoracion1.unwrap_or_default())
            .sum();
        result.header.suma_valoracion2 = result
            .entries
            .iter()
            .map(|x| x.valoracion2.unwrap_or_default())
            .sum();
        result
    }

    fn from_path(path: &Path) -> Modelo720 {
        let mut reader = Reader::from_file(path)
            .unwrap()
            .width(500)
            .linebreak(fixed_width::LineBreak::Newline);
        let mut actual_reader = reader.byte_reader();
        let header = actual_reader
            .next()
            .and_then(|x| fixed_width::from_bytes(&x.unwrap()).ok());
        let mut tipo_2_entries: Vec<Registro2Modelo720> = Vec::new();
        while let Some(entry) = actual_reader.next() {
            let entry_tipo_2 = entry
                .ok()
                .map(|x| fixed_width::from_bytes(&x).unwrap())
                .unwrap();
            tipo_2_entries.push(entry_tipo_2);
        }
        Modelo720 {
            header: header.unwrap(),
            entries: tipo_2_entries,
        }
    }

    fn save_to_file(&self, path: &Path) {
        let file = File::create(path).unwrap();
        let mut writer =
            fixed_width::Writer::from_writer(file).linebreak(fixed_width::LineBreak::Newline);
        writer
            .write_serialized(std::iter::once(self.header.clone()))
            .unwrap();
        writer.write_linebreak().unwrap();
        writer
            .write_serialized(self.entries.iter().cloned())
            .unwrap();
        writer.flush().unwrap();
    }
}

struct Modelo720Code {
    code: char,
    subcode: i8,
}

struct AssetDifference {
    valuation: Decimal,
    shares: Shares,
}

trait AssetWithValuation {
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

fn asset_difference(
    left: &dyn AssetWithValuation,
    right: &dyn AssetWithValuation,
) -> AssetDifference {
    AssetDifference {
        valuation: left.valuation() - right.valuation(),
        shares: Shares(left.shares().0 - right.shares().0),
    }
}

struct Etf {
    isin: String,
    euro_valuation: Decimal,
    shares: Decimal,
    deposit_country: String,
    description: String,
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

struct MintosNote {
    isin: String,
    euro_valuation: Decimal,
    // acquisition_date: NaiveDate,
    deposit_country: String,
    description: String,
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

#[derive(Default)]
struct Portfolio {
    assets: Vec<Rc<dyn AssetWithValuation>>,
}

impl Portfolio {
    fn from_assets(assets: Vec<Rc<dyn AssetWithValuation>>) -> Portfolio {
        let mut result = Portfolio { assets };
        result
            .assets
            .sort_by(|a, b| a.isin().partial_cmp(b.isin()).unwrap());
        result
    }

    fn merge(mut self, other: Portfolio) -> Self {
        self.assets.extend_from_slice(&other.assets);
        self.assets
            .sort_by(|a, b| a.isin().partial_cmp(b.isin()).unwrap());
        // TODO: Add safety check
        self
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

fn parse_ibkr_statement(path: &Path) -> std::io::Result<Portfolio> {
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

fn parse_mintos_statement(path: &Path) -> std::io::Result<Portfolio> {
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
enum SupportedBrokers {
    InteractiveBrokers,
    Mintos,
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
    let a = Modelo720::from_path(left);
    let mut b = Modelo720::from_path(right);
    let mut result = a;
    result.header.numero_registros_tipo2 += b.header.numero_registros_tipo2;
    result.header.suma_valoracion1 += b.header.suma_valoracion1;
    result.header.suma_valoracion2 += b.header.suma_valoracion2;
    result.entries.append(&mut b.entries);
    result
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
