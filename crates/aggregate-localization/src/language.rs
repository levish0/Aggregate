#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    #[default]
    Korean,
    English,
}

impl Language {
    pub fn locale(self) -> &'static str {
        match self {
            Self::Korean => "ko-KR",
            Self::English => "en-US",
        }
    }

    pub fn other(self) -> Self {
        match self {
            Self::Korean => Self::English,
            Self::English => Self::Korean,
        }
    }
}
