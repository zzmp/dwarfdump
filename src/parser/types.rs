use super::*;

impl<'file, Endian: gimli::Endianity> Parser<'file, Endian> {
    pub fn parse_type(&self, dict: &HashMap<usize, Type>, offset: usize) -> Type {
        unimplemented!()
    }
}
