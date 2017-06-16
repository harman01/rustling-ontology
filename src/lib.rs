//! General purpose ontology based on rustling.
//!
//! Contains detectors for various entities, like numbers, temperatures, dates
//! in french, english, ...
//!
//! ```
//! extern crate rustling;
//! extern crate rustling_ontology;
//!
//! fn main() {
//!     use rustling_ontology::*;
//!
//!     let ctx = ParsingContext::default();
//!     let parser = build_parser(rustling_ontology::Lang::EN).unwrap();
//!     let result = parser.parse("twenty-one", &ctx, true).unwrap();
//!
//!     let int: output::IntegerOutput= result[0].value.clone().attempt_into().unwrap();
//!     assert_eq!(21, int.0);
//! }
//! ```
extern crate rmp_serde;
extern crate serde;
#[macro_use]
extern crate serde_derive;

extern crate rustling_ontology_moment;
extern crate rustling;
extern crate rustling_ontology_rules;
extern crate rustling_ontology_values;
extern crate rustling_ontology_training as training;

pub use rustling::{AttemptInto, ParsedNode, ParserMatch, Range, Value, RustlingError,
                   RustlingResult, Sym};
pub use rustling_ontology_rules::{Lang, dims};
pub use rustling_ontology_values::dimension;
pub use rustling_ontology_values::dimension::{Dimension, DimensionKind, NumberValue};
pub use rustling_ontology_values::output;
pub use rustling_ontology_values::output::{ParsingContext, Output};
pub use rustling_ontology_moment::Interval;
pub use rustling_ontology_moment::Grain;

mod parser;

// Rustling raw parser. Don't use directly
#[doc(hidden)]
pub type RawParser = rustling::Parser<dimension::Dimension, parser::Feat, parser::FeatureExtractor>;

/// Main class to be use at runtime.
pub struct Parser(RawParser);

impl Parser {
    fn translate_values(&self,
                        input: Vec<ParserMatch<Dimension>>,
                        context: &ParsingContext)
                        -> Vec<ParserMatch<Output>> {
        input
            .into_iter()
            .filter_map(|pm| {
                context
                    .resolve(&pm.value)
                    .map(|o| {
                             ParserMatch {
                                 value: o,
                                 byte_range: pm.byte_range,
                                 char_range: pm.char_range,
                                 probalog: pm.probalog,
                                 latent: pm.latent,
                             }
                         })
            })
            .collect()
    }

    pub fn parse_with_kind_order(&self,
                                 input: &str,
                                 context: &ParsingContext,
                                 order: &[DimensionKind], 
                                 remove_overlap:bool)
                                 -> RustlingResult<Vec<ParserMatch<Output>>> {
        Ok(self.translate_values(self.0.parse_with_kind_order(input, order, remove_overlap)?, context))
    }

    pub fn parse(&self,
                 input: &str,
                 context: &ParsingContext,
                 remove_overlap:bool)
                 -> RustlingResult<Vec<ParserMatch<Output>>> {
        Ok(self.translate_values(self.0.parse(input, remove_overlap)?, context))
    }
}

/// Obtain a parser for a given language.
pub fn build_parser(lang: Lang) -> RustlingResult<Parser> {
    match lang {
        Lang::EN => en::build_parser(),
        Lang::FR => fr::build_parser(),
        Lang::ES => es::build_parser(),
        Lang::KO => ko::build_parser(),
    }
}

/// Obtain a parser for a given language.
pub fn build_raw_parser(lang: Lang) -> RustlingResult<RawParser> {
    match lang {
        Lang::EN => en::build_raw_parser(),
        Lang::FR => fr::build_raw_parser(),
        Lang::ES => es::build_raw_parser(),
        Lang::KO => ko::build_raw_parser(),
    }
}

pub fn train_parser(lang: Lang) -> RustlingResult<Parser> {
    match lang {
        Lang::EN => en::train_parser(),
        Lang::FR => fr::train_parser(),
        Lang::ES => es::train_parser(),
        Lang::KO => ko::train_parser(),
    }
}

macro_rules! lang {
    ($lang:ident, $config:ident) => {
        mod $lang {
            use rustling_ontology_rules as rules;
            use super::*;

            pub fn train_parser() -> RustlingResult<Parser> {
                let rules = rules::$config::rule_set()?;
                let exs = ::training::$lang();
                let model = ::rustling::train::train(&rules, exs, ::parser::FeatureExtractor())?;
                Ok(Parser(::rustling::Parser::new(rules, model, ::parser::FeatureExtractor())))
            }

            pub fn build_raw_parser() -> RustlingResult<::RawParser> {
                let rules = rules::$config::rule_set()?;
                let model = ::rmp_serde::decode::from_read(&include_bytes!(concat!(env!("OUT_DIR"), "/", stringify!($lang), ".rmp"))[..]).map_err(|e| format!("{:?}", e))?;
                Ok(::RawParser::new(rules, model, ::parser::FeatureExtractor()))
            }

            pub fn build_parser() -> RustlingResult<::Parser> {
                build_raw_parser().map(::Parser)
            }
        }
    }
}

lang!(en, en_config);
lang!(es, es_config);
lang!(fr, fr_config);
lang!(ko, ko_config);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_long_number_en() {
        let ctx = ParsingContext::default();
        let parser = build_parser(Lang::EN).unwrap();
        let number = "one million five hundred twenty-one thousand eighty-two";
        let result = parser.parse_with_kind_order(number, &ctx,  &[DimensionKind::Number], true).unwrap();
        let int: output::IntegerOutput = result[0].value.clone().attempt_into().unwrap();
        assert_eq!(1521082, int.0);
    }

    #[test]
    #[ignore]
    fn time_resolve_complex_train_sentence() {
        let parser = build_raw_parser(Lang::EN).unwrap();
        //        let sent = "I want a return train ticket from Bordeaux to Strasbourg, friday the 12th of May, 10:32 am to wednesday the 7th of june, 6:22 pm";
        let sent = "I want a return train ticket from Bordeaux to Strasbourg, friday the 12th of May, 10:32 am to wednesday the 7th of june, 6:22 pm".to_lowercase();
        let result = parser.candidates(&*sent, |_| Some(0)).unwrap();
        println!("{}", result.len());
        for r in &result {
            println!("{:?}", &sent[r.node.root_node.byte_range.0..r.node.root_node.byte_range.1]);
        }
        panic!();
    }
}
