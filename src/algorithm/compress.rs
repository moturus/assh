use strum::{EnumString, EnumVariantNames};

#[derive(Debug, Default, EnumString, EnumVariantNames)]
#[strum(serialize_all = "kebab-case")]
pub enum Compress {
    /// Zlib compression.
    Zlib,

    /// Zlib compression (extension name).
    #[strum(serialize = "zlib@openssh.com")]
    ZlibExt,

    /// No compression.
    #[default]
    None,
}

impl Compress {
    pub fn decompress(&self, buf: Vec<u8>) -> Vec<u8> {
        match self {
            Self::None => buf,
            Self::Zlib | Self::ZlibExt => unimplemented!(),
        }
    }

    pub fn compress(&self, buf: Vec<u8>) -> Vec<u8> {
        match self {
            Self::None => buf,
            Self::Zlib | Self::ZlibExt => unimplemented!(),
        }
    }
}