use crate::error::{Error, Result};
use crate::pdu::{AttributeElement, AttributeValue, AttributesPlusTerm, Operand, Operator, Query, RpnQuery, RpnRpnOperator, RpnStructure, Term};
use rasn::types::OctetString;

/// Query language representation
#[derive(Debug, Clone)]
pub enum QueryLanguage {
    CQL(String),
}

/// CQL AST node
#[derive(Debug, Clone)]
enum CqlNode {
    /// Simple term: index = "value"
    Term { index: String, relation: String, value: String },
    /// Binary operator: left AND right, left OR right
    BinaryOp { op: CqlOperator, left: Box<CqlNode>, right: Box<CqlNode> },
    /// Unary operator: NOT term
    UnaryOp { op: CqlOperator, operand: Box<CqlNode> },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CqlOperator {
    And,
    Or,
    Not,
}

/// CQL parser with error handling
struct CqlParser {
    input: Vec<char>,
    pos: usize,
}

impl CqlParser {
    fn new(input: &str) -> Self {
        Self {
            input: input.chars().collect(),
            pos: 0,
        }
    }

    fn parse(&mut self) -> Result<CqlNode> {
        let node = self.parse_expression()?;
        self.skip_whitespace();
        if self.pos < self.input.len() {
            return Err(Error::Protocol(format!("Unexpected character at position {}: '{}'", self.pos, self.input[self.pos])));
        }
        Ok(node)
    }

    fn parse_expression(&mut self) -> Result<CqlNode> {
        self.parse_or_expression()
    }

    fn parse_or_expression(&mut self) -> Result<CqlNode> {
        let mut left = self.parse_and_expression()?;
        self.skip_whitespace();

        while self.peek() == Some('O') && self.peek_str(3).as_deref() == Some("OR ") {
            self.consume_str("OR");
            self.skip_whitespace();
            let right = self.parse_and_expression()?;
            left = CqlNode::BinaryOp {
                op: CqlOperator::Or,
                left: Box::new(left),
                right: Box::new(right),
            };
            self.skip_whitespace();
        }

        Ok(left)
    }

    fn parse_and_expression(&mut self) -> Result<CqlNode> {
        let mut left = self.parse_not_expression()?;
        self.skip_whitespace();

        while self.peek() == Some('A') && self.peek_str(4).as_deref() == Some("AND ") {
            self.consume_str("AND");
            self.skip_whitespace();
            let right = self.parse_not_expression()?;
            left = CqlNode::BinaryOp {
                op: CqlOperator::And,
                left: Box::new(left),
                right: Box::new(right),
            };
            self.skip_whitespace();
        }

        Ok(left)
    }

    fn parse_not_expression(&mut self) -> Result<CqlNode> {
        self.skip_whitespace();
        if self.peek() == Some('N') && self.peek_str(4).as_deref() == Some("NOT ") {
            self.consume_str("NOT");
            self.skip_whitespace();
            let operand = self.parse_primary()?;
            Ok(CqlNode::UnaryOp {
                op: CqlOperator::Not,
                operand: Box::new(operand),
            })
        } else {
            self.parse_primary()
        }
    }

    fn parse_primary(&mut self) -> Result<CqlNode> {
        self.skip_whitespace();

        if self.peek() == Some('(') {
            self.consume_char('(')?;
            let node = self.parse_expression()?;
            self.skip_whitespace();
            self.consume_char(')')?;
            Ok(node)
        } else {
            self.parse_term()
        }
    }

    fn parse_term(&mut self) -> Result<CqlNode> {
        self.skip_whitespace();

        // Parse index (identifier)
        let index = self.parse_identifier()?;
        self.skip_whitespace();

        // Parse relation (default to "=" if not specified)
        let relation = if self.peek() == Some('=') {
            self.consume_char('=')?;
            "=".to_string()
        } else if self.peek_str(2).as_deref() == Some(">=") {
            self.consume_str(">=");
            ">=".to_string()
        } else if self.peek_str(2).as_deref() == Some("<=") {
            self.consume_str("<=");
            "<=".to_string()
        } else if self.peek() == Some('>') {
            self.consume_char('>')?;
            ">".to_string()
        } else if self.peek() == Some('<') {
            self.consume_char('<')?;
            "<".to_string()
        } else if self.peek_str(2).as_deref() == Some("<>") {
            self.consume_str("<>");
            "<>".to_string()
        } else {
            return Err(Error::Protocol(format!("Expected relation operator at position {}", self.pos)));
        };

        self.skip_whitespace();

        // Parse value (quoted string or unquoted string)
        let value = if self.peek() == Some('"') { self.parse_quoted_string()? } else { self.parse_unquoted_string()? };

        Ok(CqlNode::Term { index, relation, value })
    }

    fn parse_identifier(&mut self) -> Result<String> {
        self.skip_whitespace();
        let start = self.pos;

        if self.pos >= self.input.len() {
            return Err(Error::Protocol("Unexpected end of input while parsing identifier".into()));
        }

        let first = self.input[self.pos];
        if !first.is_alphabetic() && first != '_' {
            return Err(Error::Protocol(format!("Invalid identifier start character: '{}' at position {}", first, self.pos)));
        }

        self.pos += 1;

        while self.pos < self.input.len() {
            let ch = self.input[self.pos];
            if ch.is_alphanumeric() || ch == '_' || ch == '.' {
                self.pos += 1;
            } else {
                break;
            }
        }

        Ok(self.input[start..self.pos].iter().collect())
    }

    fn parse_quoted_string(&mut self) -> Result<String> {
        self.consume_char('"')?;
        let mut result = String::new();
        let mut escaped = false;

        while self.pos < self.input.len() {
            let ch = self.input[self.pos];
            self.pos += 1;

            if escaped {
                match ch {
                    'n' => result.push('\n'),
                    't' => result.push('\t'),
                    'r' => result.push('\r'),
                    '\\' => result.push('\\'),
                    '"' => result.push('"'),
                    _ => result.push(ch),
                }
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                return Ok(result);
            } else {
                result.push(ch);
            }
        }

        Err(Error::Protocol("Unterminated quoted string".into()))
    }

    fn parse_unquoted_string(&mut self) -> Result<String> {
        let start = self.pos;
        let mut in_string = false;

        while self.pos < self.input.len() {
            let ch = self.input[self.pos];
            if ch.is_whitespace() || ch == ')' || ch == '(' {
                if in_string {
                    break;
                } else {
                    self.pos += 1;
                    continue;
                }
            }
            in_string = true;
            self.pos += 1;
        }

        if !in_string {
            return Err(Error::Protocol(format!("Expected value at position {}", start)));
        }

        Ok(self.input[start..self.pos].iter().collect())
    }

    fn skip_whitespace(&mut self) {
        while self.pos < self.input.len() && self.input[self.pos].is_whitespace() {
            self.pos += 1;
        }
    }

    fn peek(&self) -> Option<char> {
        if self.pos < self.input.len() {
            Some(self.input[self.pos])
        } else {
            None
        }
    }

    fn peek_str(&self, len: usize) -> Option<String> {
        if self.pos + len <= self.input.len() {
            Some(self.input[self.pos..self.pos + len].iter().collect())
        } else {
            None
        }
    }

    fn consume_char(&mut self, expected: char) -> Result<()> {
        if self.peek() == Some(expected) {
            self.pos += 1;
            Ok(())
        } else {
            Err(Error::Protocol(format!("Expected '{}' at position {}, found '{:?}'", expected, self.pos, self.peek())))
        }
    }

    fn consume_str(&mut self, expected: &str) {
        let len = expected.len();
        if self.pos + len <= self.input.len() {
            let found: String = self.input[self.pos..self.pos + len].iter().collect();
            if found == expected {
                self.pos += len;
            }
        }
    }
}

/// Maps CQL index to BIB-1 Use attribute value
/// Common mappings for Dublin Core and other standard indexes
/// BIB-1 Use attribute values: Title=4, Author=1003, etc.
fn map_index_to_use_attribute_value(index: &str) -> i64 {
    // Dublin Core mappings to BIB-1 Use attribute values
    match index {
        "dc.title" | "title" | "t" => 4,          // Title
        "dc.creator" | "author" | "a" => 1003,    // Author
        "dc.subject" | "subject" | "s" => 21,     // Subject
        "dc.date" | "date" | "d" => 31,           // Date
        "dc.identifier" | "isbn" => 7,            // ISBN
        "dc.publisher" | "publisher" => 1018,     // Publisher
        "dc.language" | "language" => 54,         // Language
        "dc.type" | "type" => 1016,               // Type
        "dc.format" | "format" => 1017,           // Format
        "dc.description" | "description" => 62,   // Abstract
        "dc.relation" | "relation" => 1019,       // Relation
        "dc.coverage" | "coverage" => 1020,       // Coverage
        "dc.rights" | "rights" => 1021,           // Rights
        "dc.contributor" | "contributor" => 1004, // Contributor
        "dc.source" | "source" => 1015,           // Source
        // Numeric index (e.g., "1", "2", "3")
        _ => {
            if let Ok(num) = index.parse::<i64>() {
                num
            } else {
                // Default to "any" attribute (1016)
                1016
            }
        }
    }
}

/// Maps CQL relation operator to BIB-1 Relation attribute value
/// BIB-1 Relation attribute values: less than=1, less than or equal=2, equal=3, greater than or equal=4, greater than=5, not equal=6
fn map_relation_to_bib1_relation(relation: &str) -> i64 {
    match relation {
        "<" => 1,  // less than
        "<=" => 2, // less than or equal
        "=" => 3,  // equal
        ">=" => 4, // greater than or equal
        ">" => 5,  // greater than
        "<>" => 6, // not equal
        _ => 3,    // default to equal
    }
}

/// Converts CQL AST node to RPN structure
fn cql_node_to_rpn(node: CqlNode) -> Result<RpnStructure> {
    match node {
        CqlNode::Term { index, relation, value } => {
            // BIB-1 attributes: we need multiple AttributeElement
            // - Type 1 (Use): the field to search (Title=4, Author=1003, etc.)
            // - Type 2 (Relation): the relation operator (=, <, >, etc.)
            // - Type 3 (Position): position in field (default: any=3)
            // - Type 4 (Structure): structure of term (default: word=2)
            // - Type 5 (Truncation): truncation (default: right truncation=100)
            // - Type 6 (Completeness): completeness (default: incomplete=1)

            let use_value = map_index_to_use_attribute_value(&index);
            let relation_value = map_relation_to_bib1_relation(&relation);

            let mut attributes = Vec::new();

            // Use attribute (Type 1) - REQUIRED
            // Note: attribute_set is specified at RpnQuery level, not in each AttributeElement
            attributes.push(AttributeElement {
                attribute_set: None,      // attribute_set is at RpnQuery level
                attribute_type: 1.into(), // Use attribute type
                attribute_value: AttributeValue::Numeric(use_value.into()),
            });

            // Relation attribute (Type 2) - REQUIRED
            attributes.push(AttributeElement {
                attribute_set: None,      // attribute_set is at RpnQuery level
                attribute_type: 2.into(), // Relation attribute type
                attribute_value: AttributeValue::Numeric(relation_value.into()),
            });

            // Position attribute (Type 3) - default to "any" (3)
            attributes.push(AttributeElement {
                attribute_set: None,
                attribute_type: 3.into(),                           // Position attribute type
                attribute_value: AttributeValue::Numeric(3.into()), // any position
            });

            // Structure attribute (Type 4) - default to "word" (2)
            attributes.push(AttributeElement {
                attribute_set: None,
                attribute_type: 4.into(),                           // Structure attribute type
                attribute_value: AttributeValue::Numeric(2.into()), // word
            });

            // Truncation attribute (Type 5) - default to "right truncation" (100)
            // For exact match (=), use "no truncation" (100) or "right truncation" (100)
            // For other relations, use right truncation (100)
            let truncation_value = if relation == "=" {
                100 // right truncation (or could be 0 for no truncation, but 100 is more common)
            } else {
                100 // right truncation
            };
            attributes.push(AttributeElement {
                attribute_set: None,
                attribute_type: 5.into(), // Truncation attribute type
                attribute_value: AttributeValue::Numeric(truncation_value.into()),
            });

            Ok(RpnStructure::Op(Operand::AttributesPlusTerm(AttributesPlusTerm {
                attributes,
                term: Term::General(OctetString::from(value.as_bytes().to_vec())),
            })))
        }
        CqlNode::BinaryOp { op, left, right } => {
            let rpn1 = cql_node_to_rpn(*left)?;
            let rpn2 = cql_node_to_rpn(*right)?;

            let operator = match op {
                CqlOperator::And => Operator::And(()),
                CqlOperator::Or => Operator::Or(()),
                CqlOperator::Not => {
                    return Err(Error::Protocol("NOT operator must be unary, not binary".into()));
                }
            };

            Ok(RpnStructure::RpnRpnOperator(RpnRpnOperator {
                rpn1: Box::new(rpn1),
                rpn2: Box::new(rpn2),
                op: operator,
            }))
        }
        CqlNode::UnaryOp { op, operand } => {
            let rpn_operand = cql_node_to_rpn(*operand)?;

            match op {
                CqlOperator::Not => {
                    // NOT in Z39.50 RPN is typically represented as AND NOT
                    // We need to create a structure that represents "NOT operand"
                    // In Z39.50, NOT is usually combined with AND: we create a dummy "all" term
                    // and use AND NOT with the operand
                    // However, a simpler approach is to use the operand with a relation "not equal"
                    // But the standard way is to use AND NOT with a universal set
                    // For now, we'll create a structure that represents NOT by using AND NOT
                    // with a universal term (any field, any value)
                    let universal_term = RpnStructure::Op(Operand::AttributesPlusTerm(AttributesPlusTerm {
                        attributes: vec![
                            AttributeElement {
                                attribute_set: None,
                                attribute_type: 1.into(), // Use: any (1016)
                                attribute_value: AttributeValue::Numeric(1016.into()),
                            },
                            AttributeElement {
                                attribute_set: None,
                                attribute_type: 2.into(), // Relation: equal (3)
                                attribute_value: AttributeValue::Numeric(3.into()),
                            },
                        ],
                        term: Term::General(OctetString::from(b"*".as_slice())),
                    }));

                    Ok(RpnStructure::RpnRpnOperator(RpnRpnOperator {
                        rpn1: Box::new(universal_term),
                        rpn2: Box::new(rpn_operand),
                        op: Operator::AndNot(()),
                    }))
                }
                _ => Err(Error::Protocol(format!("Unsupported unary operator: {:?}", op))),
            }
        }
    }
}

impl From<QueryLanguage> for Query {
    fn from(query_language: QueryLanguage) -> Self {
        match query_language {
            QueryLanguage::CQL(query) => {
                // Try to parse and convert CQL to Query
                // If parsing fails, create a Type1 query with the raw CQL string as term
                match parse_cql_to_query(&query) {
                    Ok(q) => q,
                    Err(_) => {
                        // Fallback: create a Type1 query with the raw CQL string as term
                        let rpn_query = RpnQuery {
                            attribute_set: crate::pdu::bib1_attribute_set().unwrap(),
                            rpn: RpnStructure::Op(Operand::AttributesPlusTerm(AttributesPlusTerm {
                                attributes: vec![], // No explicit attributes
                                term: Term::General(OctetString::from(query.as_bytes().to_vec())),
                            })),
                        };
                        Query::Type1(rpn_query)
                    }
                }
            }
        }
    }
}

/// Parses CQL string and converts it to a Z39.50 Query
/// All queries are converted to Type-1 RPN with explicit BIB-1 attributes
pub fn parse_cql_to_query(cql: &str) -> Result<Query> {
    // Parse CQL and convert to RPN with explicit BIB-1 attributes
    let mut parser = CqlParser::new(cql);
    let ast = parser.parse()?;
    let rpn_structure = cql_node_to_rpn(ast)?;

    let rpn_query = RpnQuery {
        attribute_set: crate::pdu::bib1_attribute_set()?,
        rpn: rpn_structure,
    };

    Ok(Query::Type1(rpn_query))
}
