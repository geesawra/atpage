#[derive(Debug)]
pub enum Error {
    WrongComponentsAmount(usize),
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::WrongComponentsAmount(amt) => {
                write!(f, "AT URL components must be 3, found {amt}")
            }
        }
    }
}

#[allow(unused)]
pub struct ATURL {
    pub did: String,
    pub collection: String,
    pub key: String,
    pub blob: bool,
    pub needs_resolution: bool,
}

impl TryFrom<String> for ATURL {
    type Error = Error;

    fn try_from(value: String) -> Result<Self, Error> {
        let value = value.strip_prefix("at://").unwrap_or(&value);

        let comp = value.split("/").collect::<Vec<&str>>();

        if comp.len() != 3 {
            return Err(Error::WrongComponentsAmount(comp.len()));
        }

        Ok(ATURL {
            did: comp[0].to_string(),
            collection: comp[1].to_string(),
            key: comp[2].to_string(),
            blob: comp[1].to_string() == "blobs",
            needs_resolution: !comp[0].starts_with("did:"),
        })
    }
}
