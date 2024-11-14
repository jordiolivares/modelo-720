use std::iter::Sum;
use std::ops::AddAssign;
use std::path::Path;
use std::str::FromStr;
use std::{fs::File, io::Write};

use chrono::NaiveDate;
use fixed_width::Reader;
use fixed_width_derive::FixedWidth;
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;
use serde::de::Visitor;
use serde::{de, Deserialize, Serialize};

#[derive(Clone, Copy, Debug)]
pub struct Modelo720Number<const NUMBERS: usize>(Decimal);

impl<const N: usize> Modelo720Number<N> {
    pub fn rounded_to_cents(&self) -> Self {
        Modelo720Number(
            self.0
                .round_dp_with_strategy(2, rust_decimal::RoundingStrategy::MidpointAwayFromZero),
        )
    }
}

impl<const N: usize> From<Decimal> for Modelo720Number<N> {
    fn from(value: Decimal) -> Self {
        Self(value)
    }
}

impl<const N: usize> AddAssign for Modelo720Number<N> {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0
    }
}

impl<const N: usize> Sum for Modelo720Number<N> {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        let mut res = Modelo720Number(Decimal::ZERO);
        for x in iter {
            res += x;
        }
        res
    }
}

impl<const N: usize> Serialize for Modelo720Number<N> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let decimal_cents = self.rounded_to_cents().0;
        let sign = {
            if decimal_cents.is_sign_negative() {
                'N'
            } else {
                ' '
            }
        };
        let number = (decimal_cents.abs() * Decimal::from(100)).to_i64().unwrap();
        let string = format!("{sign}{number:0>width$}", width = N - 1);
        serializer.serialize_str(&string)
    }
}

struct Modelo720NumberVisitor<const N: usize>;

impl<'de, const N: usize> Visitor<'de> for Modelo720NumberVisitor<N> {
    type Value = Modelo720Number<N>;

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        let (sign, number) = {
            if v.len() == N {
                v.split_at(1)
            } else {
                (" ", v)
            }
        };
        let decimal = Decimal::from_str(number);
        let res = decimal
            .map(|n| if sign == " " { n } else { -n })
            .map(|n| n / Decimal::from(100))
            .map(Modelo720Number);
        res.map_err(|e| de::Error::custom(e.to_string()))
    }

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("Expected a valid number")
    }
}

impl<'de, const N: usize> Deserialize<'de> for Modelo720Number<N> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let visitor: Modelo720NumberVisitor<N> = Modelo720NumberVisitor;
        deserializer.deserialize_str(visitor)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Shares(pub Decimal);

impl Serialize for Shares {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let rounded_to_cents = self
            .0
            .round_dp_with_strategy(2, rust_decimal::RoundingStrategy::MidpointAwayFromZero);
        serializer.serialize_i64((rounded_to_cents * Decimal::from(100)).to_i64().unwrap())
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
pub struct Modelo720Date(pub Option<NaiveDate>);

impl Serialize for Modelo720Date {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match &self.0 {
            Some(v) => serializer.serialize_some(v.format("%Y%m%d").to_string().as_str()),
            None => serializer.serialize_none(), // This will output 00000000 into the file.
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
            // Result from serializing a None, the date is invalid anyways so it's treated as a special value.
            Ok(None)
        } else {
            NaiveDate::parse_from_str(v, "%Y%m%d")
                .map_err(|e| E::custom(e.to_string()))
                .map(Some)
        }
    }
}

#[derive(Clone, Deserialize, Serialize, Debug, FixedWidth)]
pub struct Registro1Modelo720 {
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

    #[fixed_width(name = "SUMA TOTAL DE VALORACIÓN 1", range = "144..162")]
    suma_valoracion1: Modelo720Number<{ 162 - 144 }>,

    #[fixed_width(name = "SUMA TOTAL DE VALORACIÓN 2", range = "162..180")]
    suma_valoracion2: Modelo720Number<{ 180 - 162 }>,

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
            suma_valoracion1: Modelo720Number(Decimal::ZERO),
            suma_valoracion2: Modelo720Number(Decimal::ZERO),
            blancos: String::default(),
        }
    }
}

#[derive(Clone, Debug)]
pub enum Modelo720Titularidad {
    Titular,
    Representate,
    Autorizado,
    Beneficiario,
    Usufructuario,
    Tomador,
    ConPoderDisposicion,
    Otros(String),
}

impl Serialize for Modelo720Titularidad {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Modelo720Titularidad::Titular => serializer.serialize_char('1'),
            Modelo720Titularidad::Representate => serializer.serialize_char('2'),
            Modelo720Titularidad::Autorizado => serializer.serialize_char('3'),
            Modelo720Titularidad::Beneficiario => serializer.serialize_char('4'),
            Modelo720Titularidad::Usufructuario => serializer.serialize_char('5'),
            Modelo720Titularidad::Tomador => serializer.serialize_char('6'),
            Modelo720Titularidad::ConPoderDisposicion => serializer.serialize_char('7'),
            Modelo720Titularidad::Otros(what) => {
                let serialized = format!("8{}", what.to_uppercase());
                serializer.serialize_str(&serialized)
            }
        }
    }
}

struct Modelo720TitularidadVisitor;

impl<'de> Visitor<'de> for Modelo720TitularidadVisitor {
    type Value = Modelo720Titularidad;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("Expected a valid ownership type")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        let (ownership_type, potential_str) = v.split_at(1);
        match ownership_type {
            "1" => Ok(Modelo720Titularidad::Titular),
            "2" => Ok(Modelo720Titularidad::Representate),
            "3" => Ok(Modelo720Titularidad::Autorizado),
            "4" => Ok(Modelo720Titularidad::Beneficiario),
            "5" => Ok(Modelo720Titularidad::Usufructuario),
            "6" => Ok(Modelo720Titularidad::Tomador),
            "7" => Ok(Modelo720Titularidad::ConPoderDisposicion),
            "8" => Ok(Modelo720Titularidad::Otros(potential_str.to_string())),
            _ => Err(E::custom("Invalid ownership type")),
        }
    }
}

impl<'de> Deserialize<'de> for Modelo720Titularidad {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        deserializer.deserialize_str(Modelo720TitularidadVisitor)
    }
}
// TODO: Reorganize constructor so it is less reliant on magic chars
#[derive(Clone, Deserialize, Serialize, Debug, FixedWidth)]
pub struct Registro2Modelo720 {
    #[fixed_width(range = "0..1")]
    pub tipo: i8,

    #[fixed_width(range = "1..4")]
    pub modelo_declaracion: i16,

    #[fixed_width(range = "4..8")]
    pub ejercicio: i16,

    #[fixed_width(range = "8..17")]
    pub nif_declarante: String,

    #[fixed_width(range = "17..26")]
    pub nif_declarado: String,

    #[fixed_width(name = "N.I.F. DEL REPRESENTANTE LEGAL", range = "26..35")]
    pub nif_representante_legal: Option<String>,

    #[fixed_width(
        name = "APELLIDOS Y NOMBRE, RAZÓN SOCIAL O DENOMINACIÓN DEL DECLARADO",
        range = "35..75"
    )]
    pub nombre: String,

    #[fixed_width(
        name = "CLAVE DE CONDICIÓN DEL DECLARANTE Y TIPO DE TITULARIDAD SOBRE EL BIEN O DERECHO",
        range = "75..101",
        pad_with = " ",
        justify = "left"
    )]
    pub tipo_titularidad: Modelo720Titularidad,

    #[fixed_width(name = "CLAVE TIPO DE BIEN O DERECHO", range = "101..102")]
    pub clave_tipo_bien: Option<char>,

    #[fixed_width(
        name = "SUBCLAVE DE BIEN O DERECHO",
        range = "102..103",
        justify = "right",
        pad_with = "0"
    )]
    pub subclave_tipo_bien: Option<i8>,

    #[fixed_width(name = "TIPO DE DERECHO REAL SOBRE INMUEBLE", range = "103..128")]
    pub tipo_derecho_real_sobre_inmueble: Option<String>,

    #[fixed_width(name = "CÓDIGO DE PAÍS", range = "128..130")]
    pub codigo_pais: String,

    #[fixed_width(
        name = "CLAVE DE IDENTIFICACIÓN",
        range = "130..131",
        justify = "right",
        pad_with = "0"
    )]
    pub clave_identificacion: Option<i8>,

    #[fixed_width(name = "IDENTIFICACIÓN DE VALORES", range = "131..143")]
    pub identificacion_valores: Option<String>,

    #[fixed_width(name = "CLAVE IDENTIFICACIÓN DE CUENTA", range = "143..144")]
    pub clave_identificacion_cuenta: Option<char>,

    #[fixed_width(name = "CÓDIGO BIC", range = "144..155")]
    pub codigo_bic: Option<String>,

    #[fixed_width(name = "CÓDIGO DE CUENTA", range = "155..189")]
    pub codigo_cuenta: Option<String>,

    #[fixed_width(name = "IDENTIFICACIÓN DE LA ENTIDAD", range = "189..230")]
    pub identificacion_entidad: Option<String>,

    #[fixed_width(
        name = "NÚMERO DE IDENTIFICACIÓN FISCAL EN EL PAÍS DE RESIDENCIA FISCAl",
        range = "230..250"
    )]
    pub nif_pais_residencia_fiscal: Option<String>,

    #[fixed_width(name = "NOMBRE VÍA PUBLICA Y NÚMERO DE CASA", range = "250..302")]
    pub nombre_via_publica_entidad: Option<String>,

    #[fixed_width(name = "COMPLEMENTO", range = "302..342")]
    pub complemento_entidad: Option<String>,

    #[fixed_width(name = "POBLACIÓN/CIUDAD", range = "342..372")]
    pub poblacion_entidad: Option<String>,

    #[fixed_width(name = "PROVINCIA/REGIÓN/ESTADO", range = "372..402")]
    pub provincia_entidad: Option<String>,

    #[fixed_width(name = "CÓDIGO POSTAL (ZIP CODE)", range = "402..412")]
    pub codigo_postal_entidad: Option<String>,

    #[fixed_width(name = "CÓDIGO PAÍS", range = "412..414")]
    pub codigo_pais_entidad: Option<String>,

    // @FixedFormat(format = "yyyyMMdd")
    #[fixed_width(
        name = "FECHA DE INCORPORACIÓN",
        range = "414..422",
        justify = "right",
        pad_with = "0"
    )]
    pub fecha_incorporacion: Modelo720Date,

    #[fixed_width(name = "ORIGEN DEL BIEN O DERECHO", range = "422..423")]
    pub origen_bien_derecho: Option<char>,

    // @FixedFormat(format = "yyyyMMdd")
    #[fixed_width(
        name = "FECHA DE EXTINCIÓN",
        range = "423..431",
        justify = "right",
        pad_with = "0"
    )]
    pub fecha_extincion: Modelo720Date,

    #[fixed_width(name = "Valoracion 1", range = "431..446")]
    pub valoracion1: Modelo720Number<{ 446 - 431 }>,

    #[fixed_width(name = "Valoracion 2", range = "446..461")]
    pub valoracion2: Modelo720Number<{ 461 - 446 }>,

    #[fixed_width(name = "CLAVE DE REPRESENTACIÓN DE VALORES", range = "461..462")]
    pub clave_representacion_valores: Option<char>,

    #[fixed_width(
        name = "NÚMERO DE VALORES",
        range = "462..474",
        justify = "right",
        pad_with = "0"
    )]
    pub numero_valores: Option<Shares>,

    #[fixed_width(name = "CLAVE TIPO DE BIEN INMUEBLE", range = "474..475")]
    pub clave_tipo_bien_inmueble: Option<char>,

    #[fixed_width(
        name = "PORCENTAJE DE PARTICIPACIÓN",
        range = "475..480",
        justify = "right",
        pad_with = "0"
    )]
    pub porcentaje: i64,

    #[fixed_width(name = "BLANCOS", range = "480..500")]
    pub blancos: String,
}

impl Registro2Modelo720 {
    pub fn new(ejercicio: i16, nif: String, nombre: String, codigo_pais: String) -> Self {
        Registro2Modelo720 {
            tipo: 2,
            modelo_declaracion: 720,
            ejercicio: ejercicio,
            nif_declarante: nif.clone(),
            nif_declarado: nif.clone(),
            nif_representante_legal: None,
            nombre: nombre.clone(),
            tipo_titularidad: Modelo720Titularidad::Titular,
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
            valoracion1: Modelo720Number(Decimal::ZERO),
            valoracion2: Modelo720Number(Decimal::ZERO),
            clave_representacion_valores: None,
            numero_valores: None,
            clave_tipo_bien_inmueble: None,
            porcentaje: 10000,
            blancos: String::default(),
        }
    }
}

#[derive(Debug)]
pub struct Modelo720 {
    // TODO: These should definitely be private
    pub header: Registro1Modelo720,
    pub entries: Vec<Registro2Modelo720>,
}

impl Modelo720 {
    pub fn new(
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
        result.header.suma_valoracion1 = Modelo720Number(
            result
                .entries
                .iter()
                .map(|x| x.valoracion1.rounded_to_cents())
                .sum::<Modelo720Number<15>>()
                .0,
        );
        result.header.suma_valoracion2 = Modelo720Number(
            result
                .entries
                .iter()
                .map(|x| x.valoracion2.rounded_to_cents())
                .sum::<Modelo720Number<15>>()
                .0,
        );
        result
    }

    pub fn from_path(path: &Path) -> Modelo720 {
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

    pub fn save_to_file(&self, path: &Path) {
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

    pub fn concat(&mut self, mut other: Modelo720) {
        self.header.numero_registros_tipo2 += other.header.numero_registros_tipo2;
        self.header.suma_valoracion1 += other.header.suma_valoracion1;
        self.header.suma_valoracion2 += other.header.suma_valoracion2;
        self.entries.append(&mut other.entries);
    }
}

pub struct Modelo720Code {
    pub code: char,
    pub subcode: i8,
}
