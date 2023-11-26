use std::fmt::Formatter;

pub type BotResult<T> = Result<T, BotError>;
pub type BotGroupResult<T> = Result<T, BotErrorGroup>;

#[derive(Debug, Clone, Default)]
pub struct BotErrorGroup {
    errors: Vec<BotError>,
}

impl std::fmt::Display for BotErrorGroup {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut syntax = vec![];
        let mut backend = vec![];
        for err in &self.errors {
            match err {
                BotError::Syntax { .. } => syntax.push(err),
                BotError::Backend { .. } => backend.push(err),
                BotError::FileSystem { .. } => syntax.push(err),
            }
        }
        let syntax_text = if syntax.is_empty() {
            "".to_string()
        } else {
            format!(
                "语法错误：\n----------\n{}\n----------\n",
                syntax
                    .iter()
                    .map(|s| format!("{}", s))
                    .collect::<Vec<_>>()
                    .join("\n----------\n"),
            )
        };
        let backend_text = if backend.is_empty() {
            "".to_string()
        } else {
            format!(
                "后端错误：\n----------\n{}",
                backend
                    .iter()
                    .map(|s| format!("{}", s))
                    .collect::<Vec<_>>()
                    .join("\n----------\n"),
            )
        };
        write!(
            f,
            "机器人错误：\n----------\n{}{}",
            syntax_text, backend_text
        )
    }
}

impl From<Vec<BotError>> for BotErrorGroup {
    #[inline]
    fn from(value: Vec<BotError>) -> Self {
        Self { errors: value }
    }
}

impl<T: Into<BotError>> From<T> for BotErrorGroup {
    fn from(value: T) -> Self {
        Self {
            errors: vec![value.into()],
        }
    }
}

impl AsRef<Vec<BotError>> for BotErrorGroup {
    #[inline]
    fn as_ref(&self) -> &Vec<BotError> {
        &self.errors
    }
}

impl AsMut<Vec<BotError>> for BotErrorGroup {
    #[inline]
    fn as_mut(&mut self) -> &mut Vec<BotError> {
        &mut self.errors
    }
}

#[allow(dead_code)]
impl BotErrorGroup {
    #[inline]
    pub fn new() -> BotErrorGroup {
        Self { errors: vec![] }
    }

    #[inline]
    pub fn with(v: Vec<BotError>) -> BotErrorGroup {
        Self { errors: v }
    }

    #[inline]
    pub fn errors(&self) -> &[BotError] {
        &self.errors
    }

    #[inline]
    pub fn do_with(&mut self, mut func: impl FnMut(&mut Vec<BotError>)) {
        func(&mut self.errors)
    }

    #[inline]
    pub fn push(&mut self, err: BotError) {
        self.errors.push(err);
    }

    #[inline]
    pub fn into_inner(self) -> Vec<BotError> {
        self.errors
    }
}

#[derive(Debug, Clone)]
pub enum BotError {
    Syntax {
        found: Option<String>,
        expected: Option<String>,
        note: Option<String>,
    },
    Backend {
        code: Option<reqwest::StatusCode>,
        source: String,
    },
    FileSystem {
        content: String,
    },
}

impl From<reqwest::Error> for BotError {
    fn from(err: reqwest::Error) -> Self {
        Self::Backend {
            code: err.status(),
            source: err.to_string(),
        }
    }
}

impl From<std::io::Error> for BotError {
    fn from(err: std::io::Error) -> Self {
        Self::FileSystem {
            content: err.to_string(),
        }
    }
}

impl std::fmt::Display for BotError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Syntax {
                found,
                expected,
                note,
            } => write!(
                f,
                "找到：{}\n期望：{}\n注释：{}",
                found.as_ref().unwrap_or(&"<NULL>".into()),
                expected.as_ref().unwrap_or(&"<NULL>".into()),
                note.as_ref().unwrap_or(&"<NULL>".into())
            ),
            Self::Backend { code, source } => {
                write!(
                    f,
                    "返回码：{}\n返回内容：{source}",
                    code.map(|s| s.to_string())
                        .unwrap_or("<UNREACHABLE>".into())
                )
            }
            Self::FileSystem { content } => {
                write!(f, "文件系统错误：{}", content)
            }
        }
    }
}
