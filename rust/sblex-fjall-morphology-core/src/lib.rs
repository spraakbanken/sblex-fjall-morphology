use fs_err as fs;
use std::{
    fmt,
    io::{self, BufRead},
    path::Path,
};

use fjall::{Database, Keyspace, KeyspaceCreateOptions, PersistMode};

#[derive(Debug)]
pub enum FjallMorphologyError {
    CantOpenDatabase {
        folder: String,
        err: fjall::Error,
    },
    CantCreateKeyspace(fjall::Error),
    CantReadWord {
        word: String,
        err: fjall::Error,
    },
    FailedToReadPrefixForWord {
        word: String,
        err: fjall::Error,
    },
    FailedToInsertWord {
        word: String,
        err: fjall::Error,
    },
    FailedToPersistDb(fjall::Error),
    Io(io::Error),
    JsonDeserialize {
        line: usize,
        path: String,
        err: serde_json::Error,
    },
    JsonSerialize(serde_json::Error),
}

impl From<io::Error> for FjallMorphologyError {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

impl fmt::Display for FjallMorphologyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CantCreateKeyspace(err) => {
                f.write_fmt(format_args!("failed to create keyspace: {err}"))
            }
            Self::CantOpenDatabase { folder, err } => {
                f.write_fmt(format_args!("failed to open database '{folder}': {err}"))
            }
            Self::CantReadWord { word, err } => {
                f.write_fmt(format_args!("failed to read '{word}' from db: {err}"))
            }
            Self::FailedToReadPrefixForWord { word, err } => f.write_fmt(format_args!(
                "failed to read key for prefix '{word}': {err}"
            )),
            Self::FailedToInsertWord { word, err } => f.write_fmt(format_args!(
                "failed to insert value to word '{word}': {err}"
            )),
            Self::FailedToPersistDb(err) => {
                f.write_fmt(format_args!("failed to persist database: {err}"))
            }
            Self::Io(err) => f.write_fmt(format_args!("{}", err)),
            Self::JsonDeserialize { line, path, err } => f.write_fmt(format_args!(
                "failed to deserialize JSON from line {line} of '{path}': {err}"
            )),
            Self::JsonSerialize(err) => {
                f.write_fmt(format_args!("failed to serialize JSON: {err}"))
            }
        }
    }
}

impl std::error::Error for FjallMorphologyError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::CantCreateKeyspace(err) => Some(err),
            Self::CantOpenDatabase { folder: _, err } => Some(err),
            Self::CantReadWord { word: _, err } => Some(err),
            Self::FailedToInsertWord { word: _, err } => Some(err),
            Self::FailedToPersistDb(err) => Some(err),
            Self::FailedToReadPrefixForWord { word: _, err } => Some(err),
            Self::Io(err) => Some(err),
            Self::JsonDeserialize {
                line: _,
                path: _,
                err,
            } => Some(err),
            Self::JsonSerialize(err) => Some(err),
        }
    }
}

#[derive(Debug, serde::Deserialize)]
struct MorphValueInRef<'a> {
    word: &'a str,
    head: &'a str,
    pos: &'a str,
    param: &'a str,
    inhs: Vec<&'a str>,
    id: &'a str,
    p: &'a str,
    #[allow(unused)]
    attr: &'a str,
}

#[derive(Debug, serde::Serialize)]
struct MorphStoredValueRef<'a> {
    gf: &'a str,
    id: &'a str,
    pos: &'a str,
    is: Vec<&'a str>,
    msd: &'a str,
    p: &'a str,
}

pub struct FjallMorphology {
    db: Database,
    saldo_morph: Keyspace,
}

impl FjallMorphology {
    pub fn new<P: AsRef<Path>>(folder: P) -> Result<Self, FjallMorphologyError> {
        let db = Database::builder(folder.as_ref()).open().map_err(|err| {
            FjallMorphologyError::CantOpenDatabase {
                folder: folder.as_ref().display().to_string(),
                err,
            }
        })?;
        let saldo_morph = db
            .keyspace("saldo_morph", KeyspaceCreateOptions::default)
            .map_err(FjallMorphologyError::CantCreateKeyspace)?;
        Ok(Self { db, saldo_morph })
    }

    pub fn build_from_path(&mut self, path: &str) -> Result<(), FjallMorphologyError> {
        let file = fs::File::open(path)?;
        let reader = io::BufReader::new(file);
        for (line_number, line) in reader.lines().enumerate() {
            let line = line?;
            let j: MorphValueInRef<'_> = serde_json::from_str(&line).map_err(|err| {
                FjallMorphologyError::JsonDeserialize {
                    line: line_number,
                    path: path.to_string(),
                    err,
                }
            })?;

            let a = MorphStoredValueRef {
                gf: j.head,
                id: j.id,
                pos: j.pos,
                is: j.inhs,
                msd: j.param,
                p: j.p,
            };
            let value = serde_json::to_string(&a).map_err(FjallMorphologyError::JsonSerialize)?;
            self.insert(j.word, value)?;
        }
        Ok(())
    }
    pub fn insert(&mut self, word: &str, value: String) -> Result<(), FjallMorphologyError> {
        let value = if let Some(data) =
            self.saldo_morph
                .get(word)
                .map_err(|err| FjallMorphologyError::CantReadWord {
                    word: word.to_string(),
                    err,
                })? {
            let mut new_value = data[..(data.len() - 1)].to_vec();
            new_value.push(b',');
            new_value.extend(value.as_bytes());
            new_value.push(b']');
            new_value
        } else {
            let mut new_value = b"[".to_vec();
            new_value.extend(value.as_bytes());
            new_value.push(b']');
            new_value
        };
        self.saldo_morph.insert(word, value).map_err(|err| {
            FjallMorphologyError::FailedToInsertWord {
                word: word.to_string(),
                err,
            }
        })?;
        self.db
            .persist(PersistMode::SyncAll)
            .map_err(FjallMorphologyError::FailedToPersistDb)?;
        Ok(())
    }
    pub fn lookup(&self, fragment: &str) -> Result<Option<Vec<u8>>, FjallMorphologyError> {
        Ok(self
            .saldo_morph
            .get(fragment)
            .map_err(|err| FjallMorphologyError::CantReadWord {
                word: fragment.to_string(),
                err,
            })?
            .map(|bytes| bytes.to_vec()))
    }

    pub fn lookup_with_cont(&self, fragment: &str) -> Result<Vec<u8>, FjallMorphologyError> {
        let mut conts: String = String::new();
        for kvpair in self.saldo_morph.prefix(fragment) {
            let key =
                kvpair
                    .key()
                    .map_err(|err| FjallMorphologyError::FailedToReadPrefixForWord {
                        word: fragment.to_string(),
                        err,
                    })?;
            let key_str = std::str::from_utf8(&key).unwrap();
            if let Some(cont) = key_str.strip_prefix(fragment)
                && let Some(c) = cont.chars().next()
                && !conts.contains(c)
            {
                conts.push(c);
            }
        }
        let mut result = b"{\"a\":".to_vec();
        if let Some(a) =
            self.saldo_morph
                .get(fragment)
                .map_err(|err| FjallMorphologyError::CantReadWord {
                    word: fragment.to_string(),
                    err,
                })?
        {
            result.extend(a.iter());
        } else {
            result.extend(b"[]");
        }
        result.extend(b",\"c\":\"");
        result.extend(conts.as_bytes());
        result.extend(b"\"}");
        Ok(result)
    }
}
