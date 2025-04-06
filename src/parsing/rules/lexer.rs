use super::*;

pub enum RuleToken {}

pub struct RuleLexer<R: Read> {
    reader: Reader<R>,
}

impl<R: Read> RuleLexer<R> {
    pub fn new(reader: &mut Reader<R>) -> Self {
        todo!()
    }

    pub fn next(&mut self) -> io::Result<Option<RuleToken>> {
        let c = loop {
            let char = self.reader.next()?;

            if char.is_none() {
                return Ok(None);
            }

            if char.unwrap() != b'\n' {
                break char.unwrap();
            }
        };

        match c as char {
            // C code to put in the final file
            ' ' => todo!(),

            // State
            '<' => todo!(),

            // List of REGEX + ACTION
            '{' => todo!(),

            // if %%: End of section
            // else if %{: C code to put in the final file
            // else simple character
            '%' => todo!(),

            // simple character
            _ => {}
        }

        todo!()
    }
}

impl<R: Read> Iterator for RuleLexer<R> {
    type Item = io::Result<RuleToken>;

    fn r#next(&mut self) -> Option<Self::Item> {
        match self.next() {
            Ok(Some(token)) => Some(Ok(token)),

            Ok(None) => None,

            Err(err) => Some(Err(err)),
        }
    }
}
