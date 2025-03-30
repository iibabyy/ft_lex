use super::*;

pub enum RuleToken {}

pub struct RuleLexer<R: Read> {
	reader: Reader<R>
}

impl<R: Read> RuleLexer<R> {
	pub fn new(reader: &mut Reader<R>) -> Self {
		todo!()
	}

	pub fn next(&mut self) -> io::Result<Option<RuleToken>> {

		while let Some(c) = self.reader.next()? {
			match c {



				_ => { todo!() }
			}
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
