//! IBGE metadata: state codes, municipality codes (sampled subset for tests).

pub const UF_AC: &str = "AC";
pub const UF_AL: &str = "AL";
pub const UF_AP: &str = "AP";
pub const UF_AM: &str = "AM";
pub const UF_BA: &str = "BA";
pub const UF_CE: &str = "CE";
pub const UF_DF: &str = "DF";
pub const UF_ES: &str = "ES";
pub const UF_GO: &str = "GO";
pub const UF_MA: &str = "MA";
pub const UF_MT: &str = "MT";
pub const UF_MS: &str = "MS";
pub const UF_MG: &str = "MG";
pub const UF_PA: &str = "PA";
pub const UF_PB: &str = "PB";
pub const UF_PR: &str = "PR";
pub const UF_PE: &str = "PE";
pub const UF_PI: &str = "PI";
pub const UF_RJ: &str = "RJ";
pub const UF_RN: &str = "RN";
pub const UF_RS: &str = "RS";
pub const UF_RO: &str = "RO";
pub const UF_RR: &str = "RR";
pub const UF_SC: &str = "SC";
pub const UF_SP: &str = "SP";
pub const UF_SE: &str = "SE";
pub const UF_TO: &str = "TO";

pub const BRAZIL: &str = "BR";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Municipality {
    pub code: &'static str,
    pub name: &'static str,
    pub state: &'static str,
}

pub const SAMPLE_MUNICIPALITIES: &[Municipality] = &[
    Municipality { code: "3550308", name: "Sao Paulo", state: UF_SP },
    Municipality { code: "3304557", name: "Rio de Janeiro", state: UF_RJ },
    Municipality { code: "5300108", name: "Brasilia", state: UF_DF },
    Municipality { code: "2927408", name: "Salvador", state: UF_BA },
    Municipality { code: "2304400", name: "Fortaleza", state: UF_CE },
    Municipality { code: "3106200", name: "Belo Horizonte", state: UF_MG },
    Municipality { code: "1302603", name: "Manaus", state: UF_AM },
    Municipality { code: "4106902", name: "Curitiba", state: UF_PR },
    Municipality { code: "2611606", name: "Recife", state: UF_PE },
    Municipality { code: "5208707", name: "Goiania", state: UF_GO },
    Municipality { code: "1501402", name: "Belem", state: UF_PA },
    Municipality { code: "3304904", name: "Sao Goncalo", state: UF_RJ },
    Municipality { code: "4314902", name: "Porto Alegre", state: UF_RS },
    Municipality { code: "3509502", name: "Campinas", state: UF_SP },
    Municipality { code: "2111300", name: "Sao Luis", state: UF_MA },
];

pub fn find_municipality(code: &str) -> Option<&'static Municipality> {
    SAMPLE_MUNICIPALITIES.iter().find(|m| m.code == code)
}

pub fn find_municipality_by_name(name: &str, state: &str) -> Option<&'static Municipality> {
    SAMPLE_MUNICIPALITIES
        .iter()
        .find(|m| m.name.eq_ignore_ascii_case(name) && m.state == state)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lookup_municipality() {
        let m = find_municipality("3550308").expect("Sao Paulo registered");
        assert_eq!(m.state, UF_SP);
        assert!(find_municipality("9999999").is_none());
    }

    #[test]
    fn lookup_municipality_by_name() {
        let m = find_municipality_by_name("Curitiba", UF_PR).expect("Curitiba registered");
        assert_eq!(m.code, "4106902");
    }
}
