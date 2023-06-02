#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    strum::EnumString,
    strum::Display,
    strum::AsRefStr,
    strum::IntoStaticStr,
    strum::EnumVariantNames,
)]
#[strum(ascii_case_insensitive, serialize_all = "lowercase")]
pub enum Datepart {
    Year,
    Month,
    Day,
}
